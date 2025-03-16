use std::{path::Path, process::exit, u64};

use arg_parser::{FlagArg, Flags};
use colorize::AnsiColor;
use constant::{DEFAULT_CLOCK_SPEED, NAME};
// use cpu::Runtime;
use cpu::{VMError, VMErrorCode};
// use shell::Shell;
// main.rs
//
mod constant;
// mod cpu;
mod arg_parser;
mod cpu;
mod isa;
mod memory;
mod mmio;
mod opcode;
mod shell;
static mut VERBOSE_FLAG: usize = 0;
static mut DISASSEMBLE: bool = false;
static mut VERY_VERBOSE_FLAG: bool = false;
static mut VERY_VERY_VERBOSE_FLAG: bool = false;
static mut INPUT_FLAG: bool = false;
static mut OUTPUT_FLAG: bool = false;
// static mut CLOCK_SPEED_MS: usize = 5; //ms
static mut GLOBAL_CLOCK: usize = 0000;

enum DisplayMode {
    Window,
    Stdout,
}
fn main() -> Result<(), VMError> {
    let cli_args: Vec<String> = std::env::args().collect();
    let flag_definitions = &[
        FlagArg::new("shell", 's', 0),
        FlagArg::new("verbose", 'v', 0),
        FlagArg::new("disassemble", 'd', 0),
        FlagArg::new("input", 'i', 0),
        FlagArg::new("output", 'o', 0),
        FlagArg::new("clock-speed", 'c', 1),
        FlagArg::new("display", 'D', 1),
    ];
    let flags = Flags::new(flag_definitions);
    let parsed_args = match arg_parser::ParsedCLIArgs::parse_arguments(&flags, &cli_args) {
        Ok(args) => args,
        Err(why) => return Err(VMError::new(VMErrorCode::CLIArgError, why)),
    };

    let mut file: String = if let Some(f) = parsed_args.raw.get(0) {
        f.to_string()
    } else {
        return Err(VMError::new(
            VMErrorCode::CLIArgError,
            "no input file".to_string(),
        ));
    };
    let mut is_shell_instance = false;
    let mut clock_speed_hz = DEFAULT_CLOCK_SPEED;
    let mut display = DisplayMode::Window;
    for arg in parsed_args.flags {
        match arg.name {
            "shell" => is_shell_instance = true,
            "verbose" => unsafe { VERBOSE_FLAG += 1 },
            "disassemble" => unsafe { DISASSEMBLE = true },
            "input" => unsafe { INPUT_FLAG = true },
            "output" => unsafe { OUTPUT_FLAG = true },
            "display" => {
                let mode_arg = arg.data[0];
                display = match mode_arg {
                    "window" => DisplayMode::Window,
                    "stdout" => DisplayMode::Stdout,
                    _ => {
                        return Err(VMError::new(
                            VMErrorCode::CLIArgError,
                            format!(
                            "{mode_arg} is not a valid mode of {}, available are window and stdout",
                            arg.name
                        ),
                        ))
                    }
                };
            }
            "clock-speed" => {
                clock_speed_hz = match arg.data[0].parse() {
                    Ok(hz) => hz,
                    Err(why) => {
                        return Err(VMError::new(
                            VMErrorCode::CLIArgError,
                            format!("invalid clock speed {} :: {}", arg.data[0], why),
                        ))
                    }
                }
            }
            _ => panic!("invalid argument snuck past parser"),
        }
    }
    let mut vm = cpu::CPU::new(clock_speed_hz, display)?;

    vm.load(&file)?;
    if is_shell_instance {
        vm.debug_shell()?;
    } else {
        vm.exec()?;
    }
    Ok(())
}

fn handle_fatal_vm_err(err: VMError) -> ! {
    println!("{err}");
    exit(1)
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
