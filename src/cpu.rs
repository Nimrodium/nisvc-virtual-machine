// cpu.rs
//

use std::path::Path;

use crate::memory::Memory;

pub struct GeneralPurposeRegisters {
    r1: u64,
    r2: u64,
    r3: u64,
    r4: u64,
    r5: u64,
    r6: u64,
    r7: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    r16: u64,
    r17: u64,
    r18: u64,
    r19: u64,
    r20: u64,
}

pub struct SpecialPurposeRegisters {
    pc: u64,
    sp: u64,

    o1: u64,
    o2: u64,
    o3: u64,
    o4: u64,
    o5: u64,
    o6: u64,
    o7: u64,
    o8: u64,
    o9: u64,
    o10: u64,
}

impl GeneralPurposeRegisters {
    fn new() -> Self {
        GeneralPurposeRegisters {
            r1: 0xDEADBEEF,
            r2: 0xDEADBEEF,
            r3: 0xDEADBEEF,
            r4: 0xDEADBEEF,
            r5: 0xDEADBEEF,
            r6: 0xDEADBEEF,
            r7: 0xDEADBEEF,
            r8: 0xDEADBEEF,
            r9: 0xDEADBEEF,
            r10: 0xDEADBEEF,
            r11: 0xDEADBEEF,
            r12: 0xDEADBEEF,
            r13: 0xDEADBEEF,
            r14: 0xDEADBEEF,
            r15: 0xDEADBEEF,
            r16: 0xDEADBEEF,
            r17: 0xDEADBEEF,
            r18: 0xDEADBEEF,
            r19: 0xDEADBEEF,
            r20: 0xDEADBEEF,
        }
    }
}
impl SpecialPurposeRegisters {
    fn new() -> Self {
        SpecialPurposeRegisters {
            pc: 0xDEADBEEF,
            sp: 0xDEADBEEF,
            o1: 0xDEADBEEF,
            o2: 0xDEADBEEF,
            o3: 0xDEADBEEF,
            o4: 0xDEADBEEF,
            o5: 0xDEADBEEF,
            o6: 0xDEADBEEF,
            o7: 0xDEADBEEF,
            o8: 0xDEADBEEF,
            o9: 0xDEADBEEF,
            o10: 0xDEADBEEF,
        }
    }
}
enum State {
    NoProgramLoaded,
    ProgramLoadedNotStarted,
    ProgramRunning,
    ProgramFatalError,
    ProgramExitedSuccess,
    ProgramHalted,
}
pub struct Runtime {
    gpr: GeneralPurposeRegisters,
    spr: SpecialPurposeRegisters,
    memory: Memory,
    state: State,
    debug: bool,
}
/// init a new runtime with program loaded
impl Runtime {
    pub fn new_loaded(binary: &Path, debug: bool) -> Result<Self, String> {
        let memory = match Memory::load(binary) {
            Ok(memory) => memory,
            Err(why) => return Err(why),
        };
        let state = State::ProgramLoadedNotStarted;
        Ok(Runtime {
            memory,
            gpr: GeneralPurposeRegisters::new(),
            spr: SpecialPurposeRegisters::new(),
            state,
            debug,
        })
    }
    /// always successful init runtime with no program loaded
    pub fn new_unloaded(debug: bool) -> Self {
        Runtime {
            memory: Memory::new_uninit(),
            gpr: GeneralPurposeRegisters::new(),
            spr: SpecialPurposeRegisters::new(),
            state: State::NoProgramLoaded,
            debug,
        }
    }
    pub fn load(&mut self, binary: &Path) -> Result<(), String> {
        self.memory = Memory::load(binary)?;
        self.state = State::ProgramLoadedNotStarted;
        Ok(())
    }

    /// execute runtime at PC
    pub fn exec(&mut self) -> Result<(), String> {
        // program loop
        match self.state {
            State::NoProgramLoaded => return Err("cannot start execution, no program loaded".to_string()),
            State::ProgramFatalError => return Err("cannot start execution, program suffered fatal error, further execution is unpredictable".to_string()),
            State::ProgramExitedSuccess => return Err("cannot start execution, program finished execution".to_string()),
            State::ProgramRunning => return Err("cannot start execution, program already running. how did you get here?!?!".to_string()),

            _ => {
                println!("executing...");
            },
        };

        loop {
            match self.state {
                State::ProgramRunning => (),
                _ => break,
            }
            match self.step() {
                Ok(()) => (),
                Err(why) => {
                    let error = format!("error in execution :: {}", why);
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    /// step through one cycle
    pub fn step(&mut self) -> Result<(), String> {
        todo!()
    }

    /// dumps state
    pub fn dump(self) -> String {
        todo!()
    }
    pub fn throw_runtime_error(self, why: &str) {
        println!("runtime error!! :: {}", why)
    }
}
