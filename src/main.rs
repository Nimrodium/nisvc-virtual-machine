use std::{path::Path, process::exit};

use colorize::AnsiColor;
use constant::NAME;
// use cpu::Runtime;
use cpu::{VMError, VMErrorCode};
// use shell::Shell;
// main.rs
//
mod assembler;
mod constant;
// mod cpu;
mod cpu;
mod memory;
mod mmio;
mod opcode;
// mod shell;
static mut VERBOSE_FLAG: bool = false;
static mut DISASSEMBLE: bool = false;
static mut VERY_VERBOSE_FLAG: bool = false;
static mut VERY_VERY_VERBOSE_FLAG: bool = false;
fn handle_fatal_vm_err(err: VMError) -> ! {
    println!("{err}");
    exit(1)
}
fn _log_disassembly(msg: &str) {
    unsafe {
        if DISASSEMBLE {
            println!("{NAME}: {} {}", "disassembled:".green(), msg)
        }
    }
}
fn _verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG {
            println!("{NAME}: {} {}", "verbose:".yellow(), msg)
        }
    }
}
fn _very_verbose_println(msg: &str) {
    unsafe {
        if VERY_VERBOSE_FLAG {
            println!("{NAME}: {} {}", "very-verbose:".yellow(), msg)
        }
    }
}

fn _very_very_verbose_println(msg: &str) {
    unsafe {
        if VERY_VERY_VERBOSE_FLAG {
            println!("{NAME}: {} {}", "very-very-verbose:".yellow(), msg)
        }
    }
}
#[macro_export]
macro_rules! log_disassembly {
    ($($arg:tt)*) => (crate::_log_disassembly(&format!($($arg)*)));
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

fn main() -> Result<(), VMError> {
    let args: Vec<String> = std::env::args().collect();
    let mut file: Option<String> = None;
    let mut is_shell_instance = false;
    let mut skip = true;
    for arg in args {
        if skip {
            skip = false;
            continue; // THERE HAS TO BE A BETTER WAY
        }
        match arg.as_str() {
            "-s" | "--shell" => {
                let mut shell = is_shell_instance = true;
            }
            "-v" | "--verbose" => unsafe { VERBOSE_FLAG = true },
            "-vv" | "--very-verbose" => unsafe {
                VERY_VERBOSE_FLAG = true;
                VERBOSE_FLAG = true;
                very_verbose_println!("verbose print level 2 enabled")
            },
            "-vvv" | "--very-very-verbose" => unsafe {
                VERBOSE_FLAG = true;
                VERY_VERBOSE_FLAG = true;
                VERY_VERY_VERBOSE_FLAG = true;
                very_very_verbose_println!("verbose print level 3 enabled")
            },
            "-d" | "--disassemble" => unsafe { DISASSEMBLE = true },
            _ => {
                if file.is_none() {
                    file = Some(arg);
                }
            }
        }
    }

    let mut vm = cpu::CPU::new()?;
    //     Ok(vm) => vm,
    //     Err(err) => handle_fatal_vm_err(err),
    // };

    let f = if let Some(f) = file.clone() {
        f
    } else {
        return Err(VMError {
            code: VMErrorCode::CLIArgError,
            reason: "no input file".to_string(),
        });
    };
    let louis = cpu::register_value_from_slice(&[0, 1]);
    vm.load(&f)?;
    vm.exec()?;

    // if is_shell_instance {
    //     let mut shell = match Shell::new() {
    //         Ok(shell) => shell,
    //         Err(why) => {
    //             println!("failed to initalize shell instance :: {}", why);
    //             exit(0);
    //         }
    //     };
    //     if file.is_some() {
    //         let file_path = file.unwrap();
    //         println!("loading file {file_path}");
    //         let path = &Path::new(&file_path);
    //         match shell.runtime.load(&path) {
    //             Ok(()) => (),
    //             Err(err) => panic!("file failed to load {err}"),
    //         };
    //     }
    //     shell.start()
    // } else {
    //     let debug = true;
    //     let mut runtime = match Runtime::new(debug) {
    //         Ok(rt) => rt,
    //         Err(err) => {
    //             println!("{err}");
    //             exit(1)
    //         }
    //     };
    //     if file.is_some() {
    //         let file_path = file.unwrap();
    //         println!("loading file {file_path}");
    //         let path = &Path::new(&file_path);
    //         match runtime.load(&path) {
    //             Ok(()) => (),
    //             Err(err) => panic!("file failed to load {err}"),
    //         };
    //         match runtime.exec() {
    //             Ok(()) => (()),
    //             Err(why) => println!("err {why}"),
    //         }
    //     } else {
    //         println!("no file provided");
    //         exit(1)
    //     }
    // }
    Ok(())
}

// let mut shell = match Shell::new() {
//     Ok(shell) => shell,
//     Err(why) => {
//         println!("failed to initalize shell instance :: {}", why);
//         exit(0)
//     }
// };
// shell.start();
