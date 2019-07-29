//Original structure based on sysinfo/examples/src/simple.rs

#![allow(unused_must_use, non_upper_case_globals)]

extern crate sysinfo;
extern crate procfs;

use sysinfo::{System, SystemExt, ProcessExt, Pid};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::collections::{BTreeMap, BTreeSet};
//use std::path::Path;
//use std::ffi::OsStr;
//use std::result::Result;

fn print_help() {
    writeln!(&mut io::stdout(), "               ==       procview       ==               ");
    writeln!(&mut io::stdout(), "               ==  Available Commands  ==               ");
    writeln!(&mut io::stdout(), "========================================================");
    writeln!(&mut io::stdout(), "            help : Show Available Commands              ");
    writeln!(&mut io::stdout(), "              ps : View All Processes                   ");
    writeln!(&mut io::stdout(), "       pst <pid> : View Process Threads                 ");
    writeln!(&mut io::stdout(), "        lm <pid> : View Loaded Modules Within Process   ");
    writeln!(&mut io::stdout(), "        xp <pid> : View Executable Pages Within Process ");
    writeln!(&mut io::stdout(), " mem <pid> <loc> : View Process Memory At Location      ");
}

fn parse_input(input: &str, sys: &mut System) -> bool {
    match input.trim() {
        "help" => print_help(),
        "ps" => {
            sys.refresh_all();
            let mut ps_bst = BTreeMap::new();
            for (pid, proc_) in sys.get_process_list() {
                ps_bst.insert(pid, proc_);
            }
            for (key, val) in ps_bst.iter(){
                let val_exec_only = val.name().split_whitespace().next();
                match val_exec_only {
                    Some(name) => writeln!(&mut io::stdout(), "{}:\t{}", key, name),
                    None => writeln!(&mut io::stdout(), "{}:\t-", key)
                };
            }
        }
        e if e.starts_with("pst ") => {
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
                for ((add1, add2), name) in find_exec_pages(pid).iter() {
                    writeln!(&mut io::stdout(), "|---- {:X?} - {:X?} :\t{}", add1, add2, name);
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

fn main() {
    let mut t = System::new();
    let t_stin = io::stdin();
    let mut stin = t_stin.lock();
    let mut done = false;

    println!("Enter 'help' to get a command list.");
    while !done {
        let mut input = String::new();
        write!(&mut io::stdout(), "> ");
        io ::stdout().flush();

        stin.read_line(&mut input);
        if (&input as &str).ends_with('\n') {
            input.pop();
        }
        done = parse_input(input.as_ref(), &mut t);
    }
}