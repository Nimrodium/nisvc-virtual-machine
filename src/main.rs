// nisvc virtual machine rewrite
#![allow(static_mut_refs)]

mod bridge;
mod constant;
mod cpu;
mod loader;
mod memory;
mod opcode;
use std::fmt;

// use colorize::AnsiColor;
use cpu::CPU;
use crossterm::style::Stylize;

use crate::constant::NAME;

struct ExecutionError {
    error: String,
}

impl ExecutionError {
    fn new(error: String) -> Self {
        Self { error }
    }
    fn prepend(mut self, prelude: String) -> Self {
        self.error = prelude + self.error.as_str();
        self
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", "error >".on_red(), self.error) // make cooler
    }
}

static mut GLOBAL_CLOCK: usize = 0;

static mut DISASSEMBLE: bool = false;
static mut VERBOSE_FLAG: usize = 0;
static mut OUTPUT_FLAG: bool = false;
static mut INPUT_FLAG: bool = false;

fn main() {
    let mut cpu = CPU::new(10_000);
    match cpu.load("placeholder") {
        Ok(()) => (),
        Err(e) => println!("{e}"),
    };
}

fn _log_disassembly(msg: &str) {
    unsafe {
        if DISASSEMBLE {
            println!(
                "{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}",
                "disassembled:".green(),
                msg
            )
        }
    }
}

fn _log_output(msg: &str) {
    unsafe {
        if OUTPUT_FLAG {
            println!("{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}", "output:".blue(), msg)
        }
    }
}

fn _log_input(msg: &str) {
    unsafe {
        if INPUT_FLAG {
            println!("{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}", "input: ".blue(), msg)
        }
    }
}

fn _verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 1 {
            println!(
                "{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}",
                "verbose:".yellow(),
                msg
            )
        }
    }
}
fn _very_verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 2 {
            println!(
                "{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}",
                "very-verbose:".yellow(),
                msg
            )
        }
    }
}

fn _very_very_verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 3 {
            println!(
                "{NAME}: {GLOBAL_CLOCK:0>4x}: {} {}",
                "very-very-verbose:".yellow(),
                msg
            )
        }
    }
}
#[macro_export]
macro_rules! log_disassembly {
    ($($arg:tt)*) => (crate::_log_disassembly(&format!($($arg)*)));
}

#[macro_export]
macro_rules! log_output {
    ($($arg:tt)*) => (crate::_log_output(&format!($($arg)*)));
}

#[macro_export]
macro_rules! log_input {
    ($($arg:tt)*) => (crate::_log_input(&format!($($arg)*)));
}

#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => (crate::_verbose_println(&format!($($arg)*)));
}
#[macro_export]
macro_rules! very_verbose_println {
    ($($arg:tt)*) => (crate::_very_verbose_println(&format!($($arg)*)));
}
#[macro_export]
macro_rules! very_very_verbose_println {
    ($($arg:tt)*) => (crate::_very_very_verbose_println(&format!($($arg)*)));
}
