use crate::{
    constant,
    cpu::{GeneralPurposeRegisters, Runtime, SpecialPurposeRegisters, State},
};
use rustyline::{error::ReadlineError, history::FileHistory, DefaultEditor, Editor};
use std::{
    io::{self, Write},
    path::Path,
    process::exit,
};
// shell mode
pub struct Shell {
    runtime: Runtime,
    readline: Editor<(), FileHistory>,
}

impl Shell {
    /// constructor
    pub fn new() -> Result<Self, String> {
        let debug = true;
        let mut runtime = Runtime::new(debug);

        println!(
            ":: NIMCODE RUNTIME SHELL VERSION {} ::",
            constant::RUNTIME_VER
        );
        let readline = match DefaultEditor::new() {
            Ok(readline) => readline,
            Err(why) => return Err(why.to_string()),
        };
        Ok(Shell { runtime, readline })
    }
    /// start/resume shell
    pub fn start(&mut self) {
        loop {
            self.prompt()
        }
    }
    /// display prompt and accept input
    pub fn prompt(&mut self) {
        let input_buffer = ("", "");
        let line = match self
            .readline
            .readline_with_initial(constant::SHELL_PROMPT, input_buffer)
        {
            Ok(line) => {
                self.readline.add_history_entry(line.as_str()).unwrap();
                line
            }
            Err(ReadlineError::Interrupted) => {
                self.cmd_exit();
            }
            Err(ReadlineError::Eof) => {
                panic!()
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        };
        let mut input = line.trim().split(" ");
        match self.decode_cmd(&mut input) {
            Ok(()) => (),
            Err(why) => println!("{}", why),
        }
        io::stdout().flush().expect("boowomp");
    }
    fn decode_cmd(&mut self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        let command_word = cmd.next().ok_or("missing command")?;
        match command_word {
            "exit" => self.cmd_exit(),
            "louis" => Ok(println!("louised")),
            "load" => self.cmd_load(cmd),
            "exec" => self.cmd_exec(cmd),
            "reset" => self.cmd_reset(cmd),
            "" => Ok(()),
            _ => Err(format!("unrecognized command [{}]", command_word)),
        }
    }
    fn cmd_exit(&mut self) -> ! {
        println!("exiting");
        exit(0);
    }
    fn cmd_load(&mut self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        let binary = Path::new(match cmd.next() {
            Some(binary) => binary,
            None => return Err("missing file in command".to_string()),
        });

        match self.runtime.load(binary) {
            Ok(()) => Ok(println!("successfully loaded binary file")),
            Err(why) => return Err(why),
        }
    }
    fn cmd_exec(&mut self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        // self.runtime.spr.pc = 0x10;
        println!("executing at PC {:#x?}", self.runtime.spr.pc);
        io::stdout().flush().expect("boowomp");
        self.runtime.exec()
    }
    fn cmd_reset(&mut self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        println!("reset runtime executable");
        self.runtime.spr = SpecialPurposeRegisters::new();
        self.runtime.gpr = GeneralPurposeRegisters::new();
        self.runtime.state = State::ProgramLoadedNotStarted;
        self.runtime.spr.pc = self.runtime.memory.start_of_exec as u64;
        Ok(())
    }
}
