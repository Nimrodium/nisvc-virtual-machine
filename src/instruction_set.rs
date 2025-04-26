use std::{mem::transmute, ops::Neg};

use crate::{
    constant::{FRAME_POINTER, PROGRAM_COUNTER, STACK_POINTER},
    cpu::{DecodedInstruction, Register, CPU},
    log_disassembly,
    memory::bytes_to_u64,
    ExecutionError,
};

impl CPU {
    pub fn op_nop(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        log_disassembly!("nop");
        Ok(())
    }

    pub fn op_cpy(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1) = Self::decode_binay_register_operation(decoded);
        let dest = self.registers.get_mut_register(dest_code)?;
        log_disassembly!("cpy {}, {}", dest.name(), op1.name());
        dest.write(op1.read());
        Ok(())
    }

    pub fn op_ldi(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let dest_code = decoded.mutable_registers[0];
        let immediate = decoded.immediates[0];
        let dest = self.registers.get_mut_register(dest_code)?;
        log_disassembly!("ldi {},${}", dest.name(), immediate);
        dest.write(immediate);
        Ok(())
    }

    pub fn op_load(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let dest_code = decoded.mutable_registers[0];
        let dest = self.registers.get_mut_register(dest_code)?;

        let n = &decoded.immutable_registers[0];
        let ptr = &decoded.immutable_registers[1];
        let value = bytes_to_u64(&self.memory.read(ptr.read(), n.read())?);
        log_disassembly!("load {}, {}, {}", dest.name(), n.name(), ptr.name());
        dest.write(value);
        Ok(())
    }
    pub fn op_store(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let ptr = &decoded.immutable_registers[0];
        let n = &decoded.immutable_registers[1];
        let value = &decoded.immutable_registers[2];
        log_disassembly!("store {}, {}, {}", ptr.name(), n.name(), value.name());
        let bytes = &value.read().to_le_bytes();
        self.memory
            .write(ptr.read(), &bytes[0..n.read() as usize])?;
        Ok(())
    }
    pub fn op_add(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() + op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("add {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }

    pub fn op_sub(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() - op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("sub {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_mult(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() * op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("mult {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_div(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() / op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("div {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_mod(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() % op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("mod {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_and(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() & op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("and {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_or(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() | op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("or {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
    pub fn op_xor(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(&decoded);
        let sum = op1.read() ^ op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("xor {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }

    pub fn op_shl(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_shr(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_rotl(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_rotr(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }

    pub fn op_void(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_void(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_void(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }
    pub fn op_void(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        Ok(())
    }

    pub fn op_not(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op) = Self::decode_binay_register_operation(decoded);
        let dest = self.registers.get_mut_register(dest_code)?;
        log_disassembly!("not {}, {}", dest.name(), op.name());
        dest.write(!op.read());
        Ok(())
    }
    pub fn op_neg(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op) = Self::decode_binay_register_operation(decoded);
        let dest = self.registers.get_mut_register(dest_code)?;
        log_disassembly!("neg {}, {}", dest.name(), op.name());
        dest.write((op.read() as i64).neg() as u64);
        Ok(())
    }
    // control flow
    pub fn op_jmp(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let address = decoded.addresses[1];
        log_disassembly!("jmp ${address}");
        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(address);
        Ok(())
    }
    pub fn op_jifz(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let condition = &decoded.immutable_registers[0];
        let address = decoded.addresses[1];
        log_disassembly!("jifz {}, ${address}", condition.name());
        if condition.read() == 0 {
            self.registers
                .get_mut_register(PROGRAM_COUNTER)?
                .write(address);
        }

        Ok(())
    }
    pub fn op_jifnz(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let condition = &decoded.immutable_registers[0];
        let address = decoded.addresses[1];
        log_disassembly!("jipub fnz {}, ${address}", condition.name());
        if condition.read() != 0 {
            self.registers
                .get_mut_register(PROGRAM_COUNTER)?
                .write(address);
        }
        Ok(())
    }
    pub fn op_inc(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let value = self
            .registers
            .get_mut_register(decoded.mutable_registers[0])?;
        log_disassembly!("inc {}", value.name());
        value.write(value.read() + 1);
        Ok(())
    }
    pub fn op_dec(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let value = self
            .registers
            .get_mut_register(decoded.mutable_registers[0])?;
        log_disassembly!("dec {}", value.name());
        value.write(value.read() - 1);
        Ok(())
    }
    pub fn op_push(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let value = &decoded.immutable_registers[0];
        log_disassembly!("push {}", value.name());
        self.push(value.read())?;
        Ok(())
    }
    pub fn op_pop(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let dest_code = decoded.mutable_registers[0];

        // let sp = self.registers.get_mut_register(STACK_POINTER)?;
        // let (new_sp, popped_value) = self.memory.pop(sp.read())?;
        // sp.write(new_sp);
        let popped_value = self.pop()?;
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(popped_value);
        log_disassembly!("pop {}", dest.name());
        Ok(())
    }
    pub fn op_call(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let address = decoded.addresses[0];
        log_disassembly!("call ${address}");

        let fp_val = self.registers.get_mut_register(FRAME_POINTER)?.read();

        self.push(fp_val)?; // old framepointer
        let sp_val = self.registers.get_register(STACK_POINTER)?.read();
        let fp = self.registers.get_mut_register(FRAME_POINTER)?;
        fp.write(sp_val - size_of::<u64>() as u64);
        let pc_val = self.registers.get_mut_register(PROGRAM_COUNTER)?.read();
        self.push(pc_val)?; // return address
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        pc.write(address);
        Ok(())
    }
    pub fn op_ret(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let return_address = self.pop()?;
        let old_fp = self.pop()?;

        let fp = self.registers.get_mut_register(FRAME_POINTER)?;
        fp.write(old_fp);

        // let sp = self.registers.get_register(STACK_POINTER)?;
        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(return_address);

        log_disassembly!("ret ${return_address}");

        todo!()
    }
    pub fn op_fopen(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let dest_code = decoded.mutable_registers[0];
        let str_ptr = &decoded.immutable_registers[0];
        let str_len = &decoded.immutable_registers[1];

        let dest = self.registers.get_mut_register(dest_code)?;
        log_disassembly!(
            "fopen {}, {}, {}",
            dest.name(),
            str_ptr.name(),
            str_len.name()
        );

        let file_path = String::from_utf8(self.memory.read(str_ptr.read(), str_len.read())?)
            .expect("fopen failed to read file name");
        let fd = self.vm_host_bridge.fopen(&file_path)?;
        dest.write(fd as u64);
        todo!()
    }

    pub fn test(&mut self, reg: &mut Register) {
        todo!()
    }
    pub fn op_fread(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_fwrite(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_fseek(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_fclose(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_malloc(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_realloc(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_free(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_memcpy(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_memset(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_itof(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_ftoi(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_fadd(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_fsub(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn op_fmult(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn op_fdiv(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        todo!()
    }
}
fn f64_as_u64(f: f64) -> u64 {
    unsafe { transmute(f) }
}

fn u64_as_f64(u: u64) -> f64 {
    unsafe { transmute(u) }
}
