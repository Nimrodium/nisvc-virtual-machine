use std::process::exit;

use shell::Shell;

// main.rs
//
mod constant;
mod cpu;
mod memory;
mod opcode;
mod shell;
fn main() {
    let mut shell = match Shell::new(None) {
        Ok(shell) => shell,
        Err(why) => {
            println!("failed to start shell instance :: {}", why);
            exit(0)
        }
    };
    shell.start();
}
