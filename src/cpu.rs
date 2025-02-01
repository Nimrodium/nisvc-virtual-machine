// cpu.rs
//

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
pub struct Register {
    gpr: GeneralPurposeRegisters,
    spr: SpecialPurposeRegisters,
}
pub struct Runtime {
    register: Register,
    memory: Memory,
}
