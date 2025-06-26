// nisvc virtual machine rewrite
#![allow(static_mut_refs)]

mod bridge;
mod constant;
mod cpu;
mod debug_shell;
mod gpu;
mod kernel;
mod loader;
mod memory;
mod opcode;
use std::fmt;

// use colorize::AnsiColor;
use crate::constant::NAME;
use clap::Parser;
use crossterm::style::Stylize;
use kernel::{Kernel, SILENCE_KERNEL};

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

static mut GLOBAL_PROGRAM_COUNTER: u64 = 0;

static mut DISASSEMBLE: bool = true;
static mut VERBOSE_FLAG: usize = 0;
static mut OUTPUT_FLAG: bool = false;
static mut INPUT_FLAG: bool = false;

#[derive(Parser)]
struct Args {
    #[arg()]
    program: String,
    #[arg(short, long, default_value_t = 0)]
    verbosity: usize,
    #[arg(short, long, default_value_t = false)]
    disassemble: bool,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    bkoffset: Option<String>,
    #[arg(long)]
    heap: Option<u64>,
    #[arg(long)]
    stack: Option<u64>,
    cmdline: Vec<String>,
}

fn main() {
    match real_main() {
        Ok(()) => (),
        Err(e) => unsafe { println!("INTERNAL FAULT @ PC{GLOBAL_PROGRAM_COUNTER}: {e}") },
    }
}

fn real_main() -> Result<(), ExecutionError> {
    let args = Args::parse();
    unsafe {
        DISASSEMBLE = args.disassemble;
        VERBOSE_FLAG = args.verbosity;
    }

    let heap = if let Some(heap) = args.heap {
        heap
    } else {
        1_000_000
    };
    let stack = if let Some(stack) = args.stack {
        stack
    } else {
        1_0000
    };
    let cmdline = {
        let mut cmdline = vec![args.program.clone()];
        cmdline.extend(args.cmdline.clone());
        cmdline
    };
    println!("cmdline: {:?}", cmdline);
    let mut kernel = Kernel::new(args.cmdline, heap, stack);
    kernel.system.load(&args.program)?;
    // kernel.gpu.as_mut().unwrap().renderer.present();
    match kernel.run() {
        Ok(()) => (),
        Err(e) => {
            // println!("stack dump:\n{:#?}", kernel.system.dump_stack());
            kernel.core_dump()?;
            println!("{e}");
        }
    };
    // kernel.core_dump()?;
    Ok(())
}

fn _log_disassembly(msg: &str) {
    unsafe {
        if DISASSEMBLE {
            println!(
                "{}: {msg}",
                format!("{GLOBAL_PROGRAM_COUNTER:0>4x}").on_dark_green()
            );
        }
    }
}

fn _kernel_log(msg: &str) {
    unsafe {
        if !SILENCE_KERNEL {
            println!(
                "{}: {}",
                format!("{GLOBAL_PROGRAM_COUNTER:0>4x} NKS:").on_dark_blue(),
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
macro_rules! kernel_log {
    ($($arg:tt)*) => (crate::_kernel_log(&format!($($arg)*)));
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
