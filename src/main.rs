use std::{path::Path, process::exit, u64};

use colorize::AnsiColor;
use constant::{DEFAULT_CLOCK_SPEED, NAME};
// use cpu::Runtime;
use cpu::{VMError, VMErrorCode};
// use shell::Shell;
// main.rs
//
mod constant;
// mod cpu;
mod cpu;
mod isa;
mod memory;
mod mmio;
mod opcode;
mod shell_;
// mod shell;
static mut VERBOSE_FLAG: usize = 0;
static mut DISASSEMBLE: bool = false;
static mut VERY_VERBOSE_FLAG: bool = false;
static mut VERY_VERY_VERBOSE_FLAG: bool = false;
static mut INPUT_FLAG: bool = false;
static mut OUTPUT_FLAG: bool = false;
// static mut CLOCK_SPEED_MS: usize = 5; //ms
static mut GLOBAL_CLOCK: usize = 0000;
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

fn main() -> Result<(), VMError> {
    let args: Vec<String> = std::env::args().collect();
    let mut file: Option<String> = None;
    let mut is_shell_instance = false;
    let mut skip = true;
    let mut clock_speed_hz = DEFAULT_CLOCK_SPEED;
    for (i, arg) in args.iter().enumerate() {
        if skip {
            skip = false;
            continue; // THERE HAS TO BE A BETTER WAY
        }
        match arg.as_str() {
            "-s" | "--shell" => {
                let mut shell = is_shell_instance = true;
            }
            "-v" | "--verbose" => unsafe { VERBOSE_FLAG = 1 },
            "-vv" | "--very-verbose" => unsafe {
                VERBOSE_FLAG = 2;
                very_verbose_println!("verbose print level 2 enabled")
            },
            "-vvv" | "--very-very-verbose" => unsafe {
                VERBOSE_FLAG = 3;

                very_very_verbose_println!("verbose print level 3 enabled")
            },
            "-d" | "--disassemble" => unsafe { DISASSEMBLE = true },
            "-i" | "--input" => unsafe { INPUT_FLAG = true },
            "-o" | "--output" => unsafe { OUTPUT_FLAG = true },
            "-io" | "-oi" => unsafe {
                INPUT_FLAG = true;
                OUTPUT_FLAG = true;
            },
            "-s" | "--shell" => is_shell_instance = true,

            "-c" | "--clock-speed" => {
                let speed_str = match args.get(i + 1) {
                    Some(a) => a,
                    None => {
                        return Err(VMError::new(
                            VMErrorCode::CLIArgError,
                            format!("missing argument for clock speed"),
                        ))
                    }
                };
                clock_speed_hz = match speed_str.parse() {
                    Ok(n) => n,
                    Err(err) => {
                        return Err(VMError::new(
                            VMErrorCode::CLIArgError,
                            format!("could not parse {speed_str} :: {err}"),
                        ))
                    }
                };
                verbose_println!("clock speed {clock_speed_hz}");
                // clock_speed_ms = 1000 / speed_hz;
            }

            _ => {
                if file.is_none() {
                    file = Some(arg.to_string());
                }
            }
        }
    }

    let mut vm = cpu::CPU::new(clock_speed_hz)?;
    //     Ok(vm) => vm,
    //     Err(err) => handle_fatal_vm_err(err),
    // };

    // r1.write(0x2222222222222222);
    // r1.write_at_quarter(0x256, 1);
    // let v255 = r1.read_at_quarter(1);
    // println!("r1 = {:#x}", r1.read());
    // println!("v255 = {v255:#x}");
    // let r1f = vm.registers._get_mut_register(0x04)?;
    // r1f.write(0xffffffffffffffff);
    // let r1 = vm.registers._get_mut_register(0x14)?;
    // println!("{}.read() -> {:#x}", r1.name(), r1._read());
    // r1.write(0x00);
    // println!("{}.read() -> {:#x}", r1.name(), r1._read());
    // let r1f = vm.registers._get_register(0x04)?;
    // println!("{}.read() -> {:#x}", r1f.name(), r1f._read());
    // exit(0);

    let f = if let Some(f) = file.clone() {
        f
    } else {
        return Err(VMError {
            code: VMErrorCode::CLIArgError,
            reason: "no input file".to_string(),
        });
    };

    vm.load(&f)?;
    if is_shell_instance {
        vm.debug_shell()?;
    } else {
        vm.exec()?;
    }
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
