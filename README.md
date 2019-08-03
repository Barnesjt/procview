# procview
This is a small project to view processes, threads, loaded modules, and executable pages of memory. It is an assignment for a class, and the absolute _first_ program I've written in __Rust__.

## The Prompt:

The assignment prompt was for a program in any programming language with these 5 basic features:

### Enumerate all the running processes.

This is implemented with the in-program command:
```
ps
```
within the program. This outputs a list sorted by PID, followed by the name of the process.

### List all the running threads within process boundary.

This is implemented with the command: 
```
pst <pid>
```
This will list all the threads for the process with the specified pid. 

### Enumerate all the loaded modules within the processes.

This is implemented with the command:
```
lm <pid>
```
This will list all the loaded modules (virtual memory with paths) for the selected pid.

### Is able to show all the executable pages within the processes.

This is implememnted with the command: 
```
xp <pid>
```
This will list all memory regions for the specified process with an executable permission set. Each region is given a number (listed at the start of the line), which can be referenced by memx.

### Gives us a capability to read the memory."

This is implememnted with 2 different commands.

First: 
```
memx <pid> <xp#>
```
This command will utilize the number listed from __xp <pid>__ to jump into the memory viewer for only the range of that executable memory. When in the memory viewer, you can use __n__ to continue viewing memory, or __q__ to quit back to the standard program.

Second, any area of available memory can be viewed with:
```
mem <pid> <addr>
```
This will take a hexadecimal address (such as one listed in the xp command). This will jump into the memory viewer, allowing you to view any memory address for any process. __WARNING:__ This command will not check address bounds for you, you can _easily_ crash the program with a bad request!

## More commands?

In addition to these commands, you can view the available commands within the main program at any time by typing __help__. This does not work from within the memory viewer.

Finally, when you want to quit, you can use the command __exit__ or __quit__. Both will close the program.

## Environment Compatability

This program was written on Ubuntu 19.04 (kernel 5.1) with Rust 1.35.0. However this program *should* work on any linux with kernel 2.6+, but I cannot verify this.

## Installing Rust to compile

Lucky you, Rust is very simple to install and use! The best way is to go to: https://www.rust-lang.org/tools/install and follow the directions.

If you are on linux (and you should be if you are trying to run this program), you will be instructed to use rustup like this:
```
curl https://sh.rustup.rs -sSf | sh
```
If everything goes well, then you should be able to run this command to see the version of Rust that is installed:
```
rustc --version
```
__NOTE:__ If this command doesn't work, then please visit the url above to get instructions about adding the correct directory to your PATH environment variable.

## How to Run

You've got Rust installed, you got've all the files from this repository (or archive) ready to go, now what? 

Navigate to the base directory (you should see cargo.toml within this directory), and run:
```
cargo build
```
This will fetch all the dependencies for this project (the external crates I used to write the program), and then build the executable.

To run the executable, you will need to give it elevated permissions to run correctly (so that is can read information about other processes). To do this run this:
```
sudo ./target/debug/procview
```
If you would like to build a release version of the executable (about 2mb), then you can use:
```
cargo build --release 
```
And the result will be at: ./target/release/procview

Thanks for checking out this project!