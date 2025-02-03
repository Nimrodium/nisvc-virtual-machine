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
            "pr-reg" => self.cmd_print_register(cmd),
            "dump" => self.cmd_dump(cmd),
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
        self.runtime.spr.pc = 0;
        Ok(())
    }
    fn cmd_print_register(&self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        match cmd.next().ok_or("missing register")?.trim() {
            "r1" => println!(
                "r1 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r1, self.runtime.gpr.r1
            ),
            "r2" => println!(
                "r2 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r2, self.runtime.gpr.r2
            ),
            "r3" => println!(
                "r3 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r3, self.runtime.gpr.r3
            ),
            "r4" => println!(
                "r4 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r4, self.runtime.gpr.r4
            ),
            "r5" => println!(
                "r5 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r5, self.runtime.gpr.r5
            ),
            "r6" => println!(
                "r6 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r6, self.runtime.gpr.r6
            ),
            "r7" => println!(
                "r7 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r7, self.runtime.gpr.r7
            ),
            "r8" => println!(
                "r8 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r8, self.runtime.gpr.r8
            ),
            "r9" => println!(
                "r9 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r9, self.runtime.gpr.r9
            ),
            "r10" => println!(
                "r10 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r10, self.runtime.gpr.r10
            ),
            "r11" => println!(
                "r11 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r11, self.runtime.gpr.r11
            ),
            "r12" => println!(
                "r12 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r12, self.runtime.gpr.r12
            ),
            "r13" => println!(
                "r13 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r13, self.runtime.gpr.r13
            ),
            "r14" => println!(
                "r14 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r14, self.runtime.gpr.r14
            ),
            "r15" => println!(
                "r15 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r15, self.runtime.gpr.r15
            ),
            "r16" => println!(
                "r16 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r16, self.runtime.gpr.r16
            ),
            "r17" => println!(
                "r17 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r17, self.runtime.gpr.r17
            ),
            "r18" => println!(
                "r18 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r18, self.runtime.gpr.r18
            ),
            "r19" => println!(
                "r19 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r19, self.runtime.gpr.r19
            ),
            "r20" => println!(
                "r20 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.gpr.r20, self.runtime.gpr.r20
            ),
            "pc" => println!(
                "pc :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.pc, self.runtime.spr.pc
            ),
            "sp" => println!(
                "sp :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.sp, self.runtime.spr.sp
            ),
            "o1" => println!(
                "o1 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o1, self.runtime.spr.o1
            ),
            "o2" => println!(
                "o2 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o2, self.runtime.spr.o2
            ),
            "o3" => println!(
                "o3 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o3, self.runtime.spr.o3
            ),
            "o4" => println!(
                "o4 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o4, self.runtime.spr.o4
            ),
            "o5" => println!(
                "o5 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o5, self.runtime.spr.o5
            ),
            "o6" => println!(
                "o6 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o6, self.runtime.spr.o6
            ),
            "o7" => println!(
                "o7 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o7, self.runtime.spr.o7
            ),
            "o8" => println!(
                "o8 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o8, self.runtime.spr.o8
            ),
            "o9" => println!(
                "o9 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o9, self.runtime.spr.o9
            ),
            "o10" => println!(
                "o10 :\ndecimal: {}\nhex: {:#x?}",
                self.runtime.spr.o10, self.runtime.spr.o10
            ),
            _ => println!("invalid register"),
        }
        Ok(())
    }
    fn cmd_dump(&self, cmd: &mut std::str::Split<'_, &str>) -> Result<(), String> {
        match cmd.next().ok_or("missing memory section")?.trim() {
            "program" => Ok(println!("program dump {:#x?}", self.runtime.memory.program)),
            "ram" => Ok(println!("ram dump {:#x?}", self.runtime.memory.ram)),
            _ => Ok(println!("invalid memory section")),
        }
    }
}
