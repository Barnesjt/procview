//Original structure based on sysinfo/examples/src/simple.rs

extern crate sysinfo;

use sysinfo::{System, SystemExt, ProcessExt};
use std::io::{self, BufRead, Write};
//use std::str::FromStr;

fn print_help() {
    writeln!(&mut io::stdout(), "Available Commands");
    writeln!(&mut io::stdout(), "help       : Show Available Commands");
    writeln!(&mut io::stdout(), "ps         : View all Processes");
    writeln!(&mut io::stdout(), "nada");
    writeln!(&mut io::stdout(), "nada");
    writeln!(&mut io::stdout(), "nada");
}

fn parse_input(input: &str, sys: &mut System) -> bool {
    match input.trim() {
        "help" => print_help(),
        "ps" => {
            for (pid, proc_) in sys.get_process_list() {
                writeln!(&mut io::stdout(), "{}:{}", pid, proc_.name());
            }
        }
        "quit" | "exit" => return true,
        e => {
            writeln!(&mut io::stdout(),"Unknown command.");
        }
    }
    false
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