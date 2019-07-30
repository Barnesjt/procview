//Original structure based on sysinfo/examples/src/simple.rs

#![allow(unused_must_use, non_upper_case_globals)]

extern crate sysinfo;
extern crate procfs;
extern crate benfred_read_process_memory;

use benfred_read_process_memory::*;
use std::convert::TryInto;
use sysinfo::{System, SystemExt, Pid};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::collections::{BTreeMap, BTreeSet};
use std::cmp;

fn print_help() {
    writeln!(&mut io::stdout(), "                  ==   procview v.0.1.0   ==                  ");
    writeln!(&mut io::stdout(), "                  ==  Available Commands  ==                  ");
    writeln!(&mut io::stdout(), "==============================================================");
    writeln!(&mut io::stdout(), "            help : Show Available Commands                    ");
    writeln!(&mut io::stdout(), "              ps : View All Processes                         ");
    writeln!(&mut io::stdout(), "       pst <pid> : View Process Threads                       ");
    writeln!(&mut io::stdout(), "        lm <pid> : View Loaded Modules Within Process         ");
    writeln!(&mut io::stdout(), "        xp <pid> : View Executable Pages Within Process       ");
    writeln!(&mut io::stdout(), "   mem <pid> <#> : View Memory of Executable Page (# from xp) ");
}

fn parse_input(input: &str, sys: &mut System) -> bool {
    match input.trim() {
        "help" => print_help(),
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
        e if e.starts_with("pst ") => {
            sys.refresh_all();
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 2 {
                writeln!(&mut io::stdout(), "pst command expects a pid argument");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                match sys.get_process(pid) {
                    Some(p) => {
                        writeln!(&mut io::stdout(), "TGID: {:?}", pid);
                        writeln!(&mut io::stdout(), "|");
                        for (key, _val) in p.tasks.iter() {
                            writeln!(&mut io::stdout(), "|--------- Thread PID: {}", key);
                        }
                    },
                    None => writeln!(&mut io::stdout(), "pid not found").expect("OK")
                };
            }
        }
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
        e if e.starts_with("xp ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 2 {
                writeln!(&mut io::stdout(), "xp command expects a pid argument");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                for (index, ((add1, add2), name)) in find_exec_pages(pid).iter().enumerate() {
                    writeln!(&mut io::stdout(), "|-- ({}) -- {:X?} - {:X?} :\t{}", index, add1, add2, name);
                }
            }
        }        
        e if e.starts_with("mem ") => {
            let tmp : Vec<&str> = e.split(' ').collect();
            if tmp.len() != 3 {
                writeln!(&mut io::stdout(), "xp command expects 2 arguments, a pid and a number (0-...)");
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
        "quit" | "exit" => return true,
        _e => {
            writeln!(&mut io::stdout(),"Unknown command.");
        }
    }
    false
}

//From read_process_memory example code
fn bytes_to_hex(bytes: &[u8]) -> String {
    let hex_bytes: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    hex_bytes.join("")
}

fn display_memory(pid: i32, add1: u64, add2: u64) {
    let handle: ProcessHandle = pid.try_into().unwrap();
    let t_stin = io::stdin();
    let mut stin = t_stin.lock();
    let mut done = false;
    let range = add2-add1;
    let mut offset = 0;
    while !done && offset < range {
        let mut chunk_size = cmp::min(400, range-offset);
        writeln!(&mut io::stdout(), "Viewing PID: {} Current Memory Addresses: {:X?} - {:X?}", pid, add1+offset, add1+offset+chunk_size);
        while chunk_size >= 40 {
            copy_address((add1+offset) as usize, cmp::min(40, chunk_size as usize), &handle)
                .map_err(|e| {
                    writeln!(&mut io::stdout(), "Error: {:?}", e);
                })
                .map(|bytes| {
                    writeln!(&mut io::stdout(), "{}", bytes_to_hex(&bytes))
                })
                .unwrap();
            offset += cmp::min(40, chunk_size);
            chunk_size -= 40;
        }

        writeln!(&mut io::stdout(), "Start Address: {:X?} Current Offset: {:X?} Ending Address: {:X?}", add1, offset, add2);

        if add1+offset+chunk_size < add2 {
            writeln!(&mut io::stdout(), "Commands: (n) to view the next page. (q) to quit memory viewer mode.");

            let mut input = String::new();
            write!(&mut io::stdout(), "$$ ");
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

fn main() {
    let mut t = System::new();
    let t_stin = io::stdin();
    let mut done = false;

    println!("Enter 'help' to get a command list.");
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