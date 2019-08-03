#! /bin/sh
# Please read the README.md for details about this project, including details about the following commands!
curl https://sh.rustup.rs -sSf | sh
cargo build
sudo ./target/debug/procview