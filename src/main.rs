//Original structure based on sysinfo/examples/src/simple.rs

#![allow(unused_must_use, non_upper_case_globals)]

//Crate Imports, procfs does most of the work, 
//sysinfo unfortunately had to be retained to easily read threads (couldn't find the ability in other crates)
extern crate sysinfo;
extern crate procfs;
extern crate benfred_read_process_memory;

//some scoping
use benfred_read_process_memory::*;
use std::convert::TryInto;
use sysinfo::{System, SystemExt, Pid};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::collections::{BTreeMap, BTreeSet};
use std::cmp;


//This function just prints the preformatted help display
fn print_help() {
    writeln!(&mut io::stdout(), "                    ==   procview v.0.1.0   ==                    ");
    writeln!(&mut io::stdout(), "                    ==  Available Commands  ==                    ");
    writeln!(&mut io::stdout(), "==================================================================");
    writeln!(&mut io::stdout(), "              help : Show Available Commands                      ");
    writeln!(&mut io::stdout(), "                ps : View All Processes                           ");
    writeln!(&mut io::stdout(), "         pst <pid> : View Process Threads                         ");
    writeln!(&mut io::stdout(), "          lm <pid> : View Loaded Modules Within Process           ");
    writeln!(&mut io::stdout(), "          xp <pid> : View Executable Pages Within Process         ");
    writeln!(&mut io::stdout(), "  mem <pid> <addr> : View Process Memory at Address               ");
    writeln!(&mut io::stdout(), "  memx <pid> <xp#> : View Memory of Executable Page (pg # from xp)");
    writeln!(&mut io::stdout(), "              quit : Close the Program                            ");
}

//This function contains the main control structure, parsing the user input give to is my main, it is also given access to
//A System variable for the sysinfo crate to use. It returns a Bool that indicates if the program should stop execution.
fn parse_input(input: &str, sys: &mut System) -> bool {
    match input.trim() {
        //help menu display
        "help" => print_help(),
        //ps displays all of the system processes in numerical order.
        //This importantly displays PID so that the remaining commands can be more easily used
        "ps" => {
            let mut ps_bst = BTreeMap::new();
            for process in procfs::all_processes() {
                ps_bst.insert(process.stat.pid, process.stat.comm);
            }
            for (key, val) in ps_bst.iter(){
                let val_exec_only = val.split_whitespace().next();
                match val_exec_only {
                    Some(name) => writeln!(&mut io::stdout(), "{}:\t{}", key, name),
                    None => writeln!(&mut io::stdout(), "{}:\t-", key)
                };
            }
        }
        //pst displays additional threads that are associated with the given PID, it does not show the main process thread again.
        e if e.starts_with("pst ") => {
            sys.refresh_all();
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 2 {
                writeln!(&mut io::stdout(), "pst command expects a pid argument");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                match sys.get_process(pid) {
                    Some(p) => {
                        writeln!(&mut io::stdout(), "TGID: {:?}", pid);
                        writeln!(&mut io::stdout(), "|---- Thread PID: {}", pid);
                        for (key, _val) in p.tasks.iter() {
                            writeln!(&mut io::stdout(), "|---- Thread PID: {}", key);
                        }
                    },
                    None => writeln!(&mut io::stdout(), "pid not found").expect("OK")
                };
            }
        }
        //lm shows the loaded modules that the process has. It uses a helper function to gather the data from a pid
        e if e.starts_with("lm ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 2 {
                writeln!(&mut io::stdout(), "lm command expects a pid argument");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                for module in find_loaded_modules(pid) {
                    writeln!(&mut io::stdout(), "|---- {}", module);
                }
            }
        }
        //xp shows all the executable parts in the processes memory, It gives address ranges and uses the string stored by the helper function to identify each area.
        //importantly, a number identifier is shown next to each range of addresses, this is used to view the memory directly with the mem command
        e if e.starts_with("xp ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 2 {
                writeln!(&mut io::stdout(), "xp command expects a pid argument");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                for (index, ((add1, add2), name)) in find_exec_pages(pid).iter().enumerate() {
                    writeln!(&mut io::stdout(), "({}) {:X?}\t{} Bytes \t{}", index, add1, (add2-add1), trim_path(name.to_string()));
                }
            }
        }
        //memx shows the actal contents of memory given a pid, and a number, as identified in the xp command
        //this command will jump into a different mode for viewing the memory, a piece at a time.       
        e if e.starts_with("memx ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 3 {
                writeln!(&mut io::stdout(), "memx command expects 2 arguments, a pid and a number (0-...)");
            } else {
                match (Pid::from_str(tmp[1]), tmp[2].parse::<usize>()) {
                    (Ok(pid), Ok(iparam)) => {
                        for (index, ((add1, add2), _name)) in find_exec_pages(pid).iter().enumerate() {
                            if iparam == index {
                                display_memory(pid, *add1, *add2);
                                return false;
                            }
                        }
                        writeln!(&mut io::stdout(), "Could not find xp {} for pid: {}. Check your arguments and try again.", iparam, pid);
                    },
                    _e => writeln!(&mut io::stdout(), "Error: <pid> and/or <xp#> are not valid numbers. Try Again.").unwrap()
                }
            }
        }
                //memx shows the actal contents of memory given a pid, and a number, as identified in the xp command
        //this command will jump into a different mode for viewing the memory, a piece at a time.       
        e if e.starts_with("mem ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 3 {
                writeln!(&mut io::stdout(), "mem command expects 2 arguments, a pid and an address");
            } else {
                match (Pid::from_str(tmp[1]), hex::decode(tmp[2])) {
                    (Ok(pid), Ok(iaddr)) => {
                        display_memory(pid, concat_vec_u8(iaddr), 18446744073699069952);
                            return false;
                    },
                    _e => writeln!(&mut io::stdout(), "Error: <pid> and/or <addr> are not valid numbers. Try Again.").unwrap()
                }
            }
        }
        //quit or exit are both valid commands to exit the program
        "quit" | "exit" => return true,
        //anything unmatched will return this error messege
        _e => {
            writeln!(&mut io::stdout(),"Unknown command.");
        }
    }
    false
}

fn concat_vec_u8 (input: Vec<u8>) -> u64 {
    concat_vec_u8_helper(0, input, 0)
}

fn concat_vec_u8_helper (count: u8, mut input: Vec<u8>, result: u64) -> u64 {
    let next = input.pop().expect("Problem encountered decoding u8 Vector");
    let modifier = 256u64.pow(count.into());
    if input.len() == 0 {
        return result + (next as u64 * modifier as u64);
    }
    concat_vec_u8_helper(count+1, input, result + (next as u64 * modifier as u64))
}

fn trim_path(orig_path: String) -> String {
    if orig_path.chars().count() > 40 {
        trim_path_helper(orig_path.to_string())
    } else {
        return orig_path;
    }
}

fn trim_path_helper(trim_path: String) -> String {
    let new_path = &trim_path[trim_path.find('/').unwrap()+1..];
    if new_path.chars().count() > 36 {
        return trim_path_helper(new_path.to_string());
    } else {
        return format!(".../{}", new_path.to_string());
    }
}

//From read_process_memory example code
fn bytes_to_hex(bytes: &[u8]) -> String {
    let hex_bytes: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    hex_bytes.join("")
}

//This is the "helper" function that does all of the memory display. It takes a pid and 2 u64s for the address range.

fn display_memory(pid: i32, add1: u64, add2: u64) {
    let handle: ProcessHandle = pid.try_into().unwrap();
    let t_stin = io::stdin();
    let mut stin = t_stin.lock();
    let mut done = false;
    //to make sure we correct track what we are displaying, we use a range variable and an offset that starts at 0
    let range = add2-add1;
    let mut offset = 0;
    while !done && offset < range {
        let mut chunk_size = cmp::min(400, range-offset);  //10 lines of 40 bytes to display per screen, chuck_size tracks the per screen amount
        writeln!(&mut io::stdout(), "Viewing PID: {} Current Memory Addresses: {:X?} - {:X?}", pid, add1+offset, add1+offset+chunk_size);
        while chunk_size >= 40 {
            copy_address((add1+offset) as usize, cmp::min(40, chunk_size as usize), &handle)
                .map_err(|e| {
                    writeln!(&mut io::stdout(), "Error: {:?}", e);
                })
                .map(|bytes| {
                    writeln!(&mut io::stdout(), "{}", bytes_to_hex(&bytes)) // is the memory is successfully read, it goes to the helper function to be displayed
                })
                .unwrap();
            offset += cmp::min(40, chunk_size);  
            chunk_size -= 40;
        }
        //I originally put this line for debugging, but I find it helpful to quick refer to
        writeln!(&mut io::stdout(), "Start Address: {:X?} Current Offset: {:X?} Ending Address: {:X?}", add1, offset, add2);

        if add1+offset+chunk_size < add2 {  //this logic lets the mem viewer automatically quit when it has displayed it's full range
            writeln!(&mut io::stdout(), "Commands: (n) to view the next page. (q) to quit memory viewer mode.");

            let mut input = String::new();
            write!(&mut io::stdout(), "$$ "); //mem viewer has a different prompt, a $$
            io ::stdout().flush();

            stin.read_line(&mut input);
            if (&input as &str).ends_with('\n') {
                input.pop();
            }
            done = read_mem_parse_input(input.as_ref());
        } else {
            writeln!(&mut io::stdout(), "Finished displaying requested memory locations.");
            done = true;
        }
    }
}

//This finds loaded modules by digging in the memory maps for anything with a path, executable or not
fn find_loaded_modules(pid: i32) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    writeln!(&mut io::stdout(), "Loaded Modules For PID: {}", pid);
    for process in procfs::all_processes() {
        if process.pid() == pid {
            match process.maps() {
              Ok(map) => {
                  for elem in &map {
                       match &elem.pathname {
                          procfs::MMapPath::Path(p) => modules.insert(p.to_str().unwrap().to_string()),
                           _e => false
                        };
                 }
              },
            _e => {}
          };
        }
    }
    modules
}

//This is similar to find_loaded_modules, except all the elements are scanned for "x" in the permissions string first.
//Each case is pattern matched to make key,value pair in a BTreeMap. We get an address range (as a tuple) for the key and a pretty informative string for the value.
fn find_exec_pages(pid: i32) -> BTreeMap<(u64, u64), String> {
    let mut exec_pages = BTreeMap::new();
    writeln!(&mut io::stdout(), "Executable Pages For PID: {}", pid);
    for process in procfs::all_processes() {
        if process.pid() == pid {
            match process.maps() {
              Ok(map) => {
                  for elem in &map {
                      if elem.perms.contains("x") {
                        match &elem.pathname {
                            procfs::MMapPath::Path(p) => exec_pages.insert(elem.address, p.to_str().unwrap().to_string()),
                            procfs::MMapPath::Heap => exec_pages.insert(elem.address, String::from("Heap")),
                            procfs::MMapPath::Stack => exec_pages.insert(elem.address, String::from("Stack")),
                            procfs::MMapPath::TStack(tid) => exec_pages.insert(elem.address, format!("Thread Stack. TID: {}", tid.to_string())),
                            procfs::MMapPath::Vdso => exec_pages.insert(elem.address, String::from("Virtual Dynamically Linked Shared Object")),
                            procfs::MMapPath::Vvar => exec_pages.insert(elem.address, String::from("Shared kernel variables")),
                            procfs::MMapPath::Vsyscall => exec_pages.insert(elem.address, String::from("Virtual syscalls")),
                            procfs::MMapPath::Anonymous => exec_pages.insert(elem.address, String::from("Anonymous/Private")),
                            procfs::MMapPath::Other(p) => exec_pages.insert(elem.address, format!("Other: {}", p))
                        }; 
                      }
                 }
              },
            _e => {}
          };
        }
    }
    exec_pages
}

//This is the little input parser for the mem viewer, returns true to quit.
fn read_mem_parse_input(input: &str) -> bool {
    match input.trim() {
        "n" => {
            return false;
        },
        "q" => {
            return true;
        },
        _e => {
            writeln!(&mut io::stdout(), "Invalid Command. Use 'n' for next and 'q' to quit.")
        }
    };
    false
}

//And finally the main function, It has a global System variable for the thread command, a bool for a flag to quit the program, and some stin input stuff.
//Note the stin is dropper after each input, so that the mem viewer isn't locked out of getting keyboard input from the user.
fn main() {
    let mut t = System::new();
    let t_stin = io::stdin();
    let mut done = false;
    //start the program by printing the help menu (which has program identification as well)
    print_help();
    //the main program loop is here, grabbing input, and quitting is parse_input returns true.
    while !done {
        let mut stin = t_stin.lock();
        let mut input = String::new();
        write!(&mut io::stdout(), "> ");
        io ::stdout().flush();

        stin.read_line(&mut input);
        drop(stin);
        if (&input as &str).ends_with('\n') {
            input.pop();
        }
        done = parse_input(input.as_ref(), &mut t);
    }
}