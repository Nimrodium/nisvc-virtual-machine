use crate::{constant, cpu::Runtime};
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
    pub fn new(binary: Option<&Path>) -> Result<Self, String> {
        let debug = true;
        let mut runtime = match binary {
            Some(path) => match Runtime::new_loaded(path, debug) {
                Ok(runtime) => runtime,
                Err(why) => {
                    println!(
                        "failed to load binary, empty runtime loaded instead :: {}",
                        why
                    );
                    Runtime::new_unloaded(debug)
                }
            },
            None => Runtime::new_unloaded(debug),
        };
        println!(
            ":: NIMCODE RUNTIME SHELL VERSION {} ::",
            constant::RUNTIME_VER
        );
        let mut readline = match DefaultEditor::new() {
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

        self.runtime.load(binary)
    }
}

pub struct Commands;
impl Commands {
    fn load(runtime: &mut Runtime) {}
    fn step(runtime: &mut Runtime) {
        match runtime.step() {
            Ok(()) => (),
            Err(runtime_err) => println!("runtime error: {}", runtime_err),
        }
    }
}
