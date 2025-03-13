use crate::{
    constant::{RegisterWidth, SHELL_PROMPT, STACK_POINTER},
    cpu::{VMError, VMErrorCode, CPU},
    verbose_println, very_verbose_println, very_very_verbose_println, INPUT_FLAG, OUTPUT_FLAG,
    VERBOSE_FLAG,
};
use rustyline::{history::FileHistory, DefaultEditor, Editor};
use std::{process::exit, str::Split};
type ShellArgs<'a> = Split<'a, &'a str>;
impl CPU {
    pub fn debug_shell(&mut self) -> Result<(), VMError> {
        verbose_println!("dropped into debug shell");
        let mut readline = match DefaultEditor::new() {
            Ok(r) => r,
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::ShellError,
                    format!("failed to initialize readline :: {why}"),
                ))
            }
        };
        loop {
            let cmd_str = self.sh_prompt(&mut readline)?;
            let mut cmd = cmd_str.split(" ");
            let command_name = match cmd.next() {
                Some(cmd) => cmd,
                None => {
                    very_very_verbose_println!("no input");
                    continue;
                }
            };
            let result: Result<(), VMError> = match command_name {
                "exit" => self.sh_exit(),
                "stop" => self.sh_stop_vm(&mut cmd),
                "rr" | "read-register" => self.sh_read_register(&mut cmd),
                "sk" | "stack" => self.sh_stack(&mut cmd),
                "s" | "step" => self.step(),
                "logctl" => self.sh_logctl(&mut cmd),
                "exec" => self.exec(),
                "rsk" => self.read_stack_from_offset(&mut cmd),
                _ => Err(VMError::new(
                    VMErrorCode::ShellCommandError,
                    format!("unrecognized command {command_name}"),
                )),
            };
            match result {
                Ok(()) => (),
                Err(err) => match err.code {
                    VMErrorCode::ShellExit => return Ok(()),
                    _ => println!("{err}"),
                    // VMErrorCode::ShellError =>
                },
            }
        }
    }

    fn sh_prompt(&mut self, readline: &mut Editor<(), FileHistory>) -> Result<String, VMError> {
        let input = match readline.readline(SHELL_PROMPT) {
            Ok(input) => {
                match readline.add_history_entry(input.as_str()) {
                    Ok(_) => (),
                    Err(_) => verbose_println!("failed to add {input} to history"),
                };
                input
            }
            Err(err) => {
                return Err(VMError::new(
                    VMErrorCode::ShellError,
                    format!("could not read line :: {err}"), // change to not crash maybe?
                ));
            }
        }
        .trim()
        .to_string();

        Ok(input)
    }
    /// exits shell
    fn sh_exit(&mut self) -> Result<(), VMError> {
        very_verbose_println!("exited shell");
        Err(VMError::new(
            VMErrorCode::ShellExit,
            "shell exit invoked".to_string(),
        ))
    }
    /// exits vm
    fn sh_stop_vm(&mut self, args: &mut ShellArgs) -> ! {
        very_verbose_println!("stopped virtual machine");
        exit(0);
    }

    fn sh_logctl(&mut self, args: &mut ShellArgs) -> Result<(), VMError> {
        // let level = match args.next() {
        //     Some(arg) => arg,
        //     None => {
        //         return Err(VMError::new(
        //             VMErrorCode::ShellCommandError,
        //             "missing register name".to_string(),
        //         ))
        //     }
        // };
        let arg = get_next_arg(args, "missing subcommand")?;
        match arg.as_str() {
            "0" => unsafe {
                println!("verbose printing disabled");
                VERBOSE_FLAG = 0
            },
            "1" => unsafe {
                println!("verbose printing set to 1");
                VERBOSE_FLAG = 1
            },
            "2" => unsafe {
                println!("verbose printing set to 2");
                VERBOSE_FLAG = 2
            },
            "3" => unsafe {
                println!("verbose printing set to 3");
                VERBOSE_FLAG = 3
            },
            "output" => unsafe {
                let on_off = get_next_arg(args, "missing mode on|off")?;
                match on_off.as_str() {
                    "on" => {
                        println!("output logging enabled");
                        OUTPUT_FLAG = true
                    }
                    "off" => {
                        println!("output logging disabled");
                        OUTPUT_FLAG = false
                    }
                    _ => {
                        return Err(VMError::new(
                            VMErrorCode::ShellCommandError,
                            format!("{arg} is not a valid output subcommand"),
                        ))
                    }
                }
            },
            "input" => unsafe {
                let on_off = get_next_arg(args, "missing mode on|off")?;
                match on_off.as_str() {
                    "on" => {
                        println!("input logging enabled");
                        INPUT_FLAG = true
                    }
                    "off" => {
                        println!("input logging disabled");
                        INPUT_FLAG = false
                    }
                    _ => {
                        return Err(VMError::new(
                            VMErrorCode::ShellCommandError,
                            format!("{arg} is not a valid input subcommand"),
                        ))
                    }
                }
            },
            _ => {
                return Err(VMError::new(
                    VMErrorCode::ShellCommandError,
                    format!("{arg} is not a valid verbosity subcommand"),
                ))
            }
        }
        Ok(())
    }

    fn sh_read_register(&mut self, args: &mut ShellArgs) -> Result<(), VMError> {
        let name = match args.next() {
            Some(arg) => arg,
            None => {
                return Err(VMError::new(
                    VMErrorCode::ShellCommandError,
                    "missing register name".to_string(),
                ))
            }
        };
        let register = self.registers.get_register_via_reverse_lookup(name)?;
        println!("{register}");
        Ok(())
    }
    fn sh_step(&mut self, args: ShellArgs) -> Result<(), VMError> {
        self.step()
    }
    fn sh_exec(&mut self, args: ShellArgs) -> Result<(), VMError> {
        verbose_println!("executing in shell");
        match self.exec() {
            Ok(_) => (),
            Err(err) => println!("{err}"),
        }
        Ok(())
    }
    fn sh_slice(&mut self, args: &mut ShellArgs) -> Result<(), VMError> {
        todo!()
    }
    fn sh_stack_slice(&mut self, args: ShellArgs) -> Result<(), VMError> {
        todo!()
    }
    //
    fn sh_get_ram_base(&mut self, args: ShellArgs) -> Result<(), VMError> {
        todo!()
    }
    // gives info of the stack's state
    fn sh_stack(&mut self, args: &mut ShellArgs) -> Result<(), VMError> {
        let position = self.registers.get_register(STACK_POINTER)?.read();
        let top_value = self.false_pop()?;
        let msg = format!(
            "stack:\n\tbase = {}\n\tmax = {}\n\tsp = {position}\n\ttop = {top_value} || {top_value:x}",
            self.stack_base, self.stack_max
        );
        println!("{msg}");
        Ok(())
    }
    /// pops from stack without permenantly altering stack pointer
    fn false_pop(&mut self) -> Result<usize, VMError> {
        let old = self.registers.get_mut_register(STACK_POINTER)?.read();
        let value = self.pop()? as usize;
        self.registers.get_mut_register(STACK_POINTER)?.write(old);
        Ok(value)
    }
    fn read_stack_from_offset(&mut self, args: &mut ShellArgs) -> Result<(), VMError> {
        let offset_str = get_next_arg(args, "missing offset")?;
        let offset: isize = match offset_str.parse() {
            Ok(offset) => offset,
            Err(err) => {
                return Err(VMError::new(
                    VMErrorCode::ShellCommandError,
                    format!("could not format {offset_str} :: {err}"),
                ))
            }
        };
        let real_stack_ptr = self.registers.get_register(STACK_POINTER)?.read();
        let target_address =
            real_stack_ptr.saturating_add_signed(offset as i64 * size_of::<RegisterWidth>() as i64);
        self.registers
            .get_mut_register(STACK_POINTER)?
            .write(target_address);
        let value = self.pop()?;
        self.registers
            .get_mut_register(STACK_POINTER)?
            .write(real_stack_ptr);
        println!("value at stack offset {offset} = {value} || {value:#x}");
        Ok(())
    }
}
fn get_next_arg(args: &mut ShellArgs, err_msg: &str) -> Result<String, VMError> {
    let arg = match args.next() {
        Some(arg) => arg,
        None => {
            return Err(VMError::new(
                VMErrorCode::ShellCommandError,
                err_msg.to_string(),
            ))
        }
    };
    Ok(arg.to_string())
}
