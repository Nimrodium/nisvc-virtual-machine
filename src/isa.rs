use crate::{
    constant::{
        RegisterWidth, ADDRESS_BYTES, FRAME_POINTER, OPCODE_BYTES, PROGRAM_COUNTER, REGISTER_BYTES,
        STACK_POINTER,
    },
    cpu::{register_value_from_slice, VMError, VMErrorCode, CPU},
    log_disassembly, log_input, log_output, verbose_println, very_verbose_println,
    very_very_verbose_println,
};
impl CPU {
    pub fn op_nop(&mut self) -> Result<usize, VMError> {
        log_disassembly!("nop");
        Ok(OPCODE_BYTES)
    }
    pub fn op_mov(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES * 2;
        let operand_bytes = self.read_operands(bytes_read)?;

        let source_register = self.registers.get_register(operand_bytes[1])?;
        let source_value = source_register.read();
        let src_reg_name = source_register.name();

        let destination_register = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!("mov {}, {src_reg_name}", destination_register.name());
        destination_register.write(source_value);

        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_movim(&mut self) -> Result<usize, VMError> {
        // 01 02 05 01
        let bytes_read = REGISTER_BYTES + 1;
        let operands = self.read_operands(bytes_read)?;
        let dest_reg_code = operands[0];
        let size = operands[1] as usize;
        let operands_with_immediate = self.read_operands(bytes_read + size)?;
        let immediate = register_value_from_slice(&operands_with_immediate[bytes_read..]);
        let dest_reg = self.registers.get_mut_register(dest_reg_code)?;
        log_disassembly!("ldi {}, ${immediate}", dest_reg.name());
        dest_reg.write(immediate);
        let total_bytes_read = OPCODE_BYTES + bytes_read + size;
        Ok(total_bytes_read)
    }
    /// load x,y,z
    /// loads bytes starting from z and extending out y bytes into rx up to x's maximum (8 bytes)
    pub fn op_load(&mut self) -> Result<usize, VMError> {
        // let (_, size, addr, bytes_read) = self.trinary_operation_decode()?;
        // cannot use dest as provided because it needs to access memory as mutable

        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let addr = self.registers.get_register(operand_bytes[2])?.clone();
        let size = self.registers.get_register(operand_bytes[1])?.clone();

        let bytes = self
            .memory
            .read_bytes(addr.read() as RegisterWidth, size.read() as usize)?;
        let value = register_value_from_slice(&bytes);
        verbose_println!("(load) {bytes:?} -> {value}");

        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        dest.write(value);
        log_disassembly!("load {}, {}, {}", dest.name(), size.name(), addr.name());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_store(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let src_reg = self.registers.get_register(operand_bytes[2])?.clone();
        let size = self.registers.get_register(operand_bytes[1])?.clone();

        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!("store {}, {}, {}", dest.name(), size.name(), src_reg.name());
        let bytes =
            &RegisterWidth::to_le_bytes(src_reg.read() as RegisterWidth)[0..size.read() as usize];
        self.memory
            .write_bytes(dest.read() as RegisterWidth, bytes)?;
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_add(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_add(op2.read()));
        log_disassembly!("add {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_sub(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_sub(op2.read()));
        log_disassembly!("sub {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_mult(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_mul(op2.read()));
        log_disassembly!("mult {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(OPCODE_BYTES + bytes_read)
    }

    pub fn op_div(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    // fn op_neg(&mut self) -> Result<usize, VMError> {
    //     todo!()
    // }
    pub fn op_or(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("or {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read() | op2.read());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_xor(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("xor {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read() ^ op2.read());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_and(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("add {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read() & op2.read());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_not(&mut self) -> Result<usize, VMError> {
        let (dest, op, bytes_read) = self.binary_operation_decode()?;
        log_disassembly!("not {}, {}", dest.name(), op.name());
        dest.write(!op.read());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_shl(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("shl {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read().wrapping_shl(op2.read() as u32));
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_shr(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("shr {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read().wrapping_shr(op2.read() as u32));
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_rotl(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("rotl {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read().rotate_left(op2.read() as u32));
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_rotr(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        log_disassembly!("rotr {}, {}, {}", dest.name(), op1.name(), op2.name());
        dest.write(op1.read().rotate_right(op2.read() as u32));
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_neg(&mut self) -> Result<usize, VMError> {
        let (dest, op, bytes_read) = self.binary_operation_decode()?;
        log_disassembly!("neg {}", op.name());
        dest.write(op.read().wrapping_neg());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_jmp(&mut self) -> Result<usize, VMError> {
        let address = register_value_from_slice(&self.read_operands(ADDRESS_BYTES)?);
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        log_disassembly!("jmp ${}", address);

        pc.write(address);
        Ok(0) // jmp moved pc so return 0 so it isnt moved again
    }
    pub fn op_jifz(&mut self) -> Result<usize, VMError> {
        let (pc, condition, address, bytes_read) = self.jif_decode()?;
        log_disassembly!("jifz {}, ${}", condition.name(), address);
        if condition.read() == 0 {
            pc.write(address);
            Ok(0)
        } else {
            Ok(OPCODE_BYTES + bytes_read)
        }
    }
    pub fn op_jifnz(&mut self) -> Result<usize, VMError> {
        let (pc, condition, address, bytes_read) = self.jif_decode()?;
        log_disassembly!("jifnz {}, ${}", condition.name(), address);
        if condition.read() != 0 {
            pc.write(address);
            Ok(0)
        } else {
            Ok(OPCODE_BYTES + bytes_read)
        }
    }
    pub fn op_pr(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    pub fn op_inc(&mut self) -> Result<usize, VMError> {
        let (dest, bytes_read) = self.unary_operation_decode()?;
        let old = dest.read();
        let new = old + 1;
        dest.write(new);
        // verbose_println!("{old}+1 = {new}||{}", dest.read());
        log_disassembly!("inc {}", dest.name());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_dec(&mut self) -> Result<usize, VMError> {
        let (dest, bytes_read) = self.unary_operation_decode()?;
        dest.write(dest.read().wrapping_sub(1));
        log_disassembly!("dec {}", dest.name());
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_push(&mut self) -> Result<usize, VMError> {
        // push does not actually require the register to be mutable
        let (register, bytes_read) = self.unary_operation_decode()?;
        let value = register.read();
        log_disassembly!("push {}", register.name());
        self.push(value)?;
        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_pop(&mut self) -> Result<usize, VMError> {
        // if there was no disassembly print this would be valid
        // let value = self.pop()?;
        // let (register, bytes_read) = self.unary_operation_decode()?;
        // register.write(value)?;

        let bytes_read = REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let register_name = self.registers.get_register(operand_bytes[0])?.name();

        log_disassembly!("pop {}", register_name);
        let value = self.pop()?;
        self.registers
            .get_mut_register(operand_bytes[0])?
            .write(value);

        Ok(OPCODE_BYTES + bytes_read)
    }
    pub fn op_call(&mut self) -> Result<usize, VMError> {
        let bytes_read = ADDRESS_BYTES;
        let subroutine_address = register_value_from_slice(&self.read_operands(bytes_read)?);
        let return_address = self.registers.get_register(PROGRAM_COUNTER)?.read()
            + OPCODE_BYTES as RegisterWidth
            + bytes_read as RegisterWidth;

        let fp_old = self.registers.get_register(FRAME_POINTER)?.read();
        self.push(fp_old)?;
        self.push(return_address)?;

        // save the frame pointer
        let sp = self.registers.get_register(STACK_POINTER)?.read();
        self.registers.get_mut_register(FRAME_POINTER)?.write(sp);

        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(subroutine_address);
        log_disassembly!("call ${subroutine_address}");
        verbose_println!("jumping to subroutine ${subroutine_address}");
        Ok(0) // manipulated program counter manually
    }
    pub fn op_ret(&mut self) -> Result<usize, VMError> {
        let return_address = self.pop()?;
        let old_fp = self.pop()?;
        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(return_address);
        self.registers
            .get_mut_register(FRAME_POINTER)?
            .write(old_fp);
        log_disassembly!("ret ${return_address}");
        Ok(0)
    }
    pub fn op_cache(&mut self) -> Result<usize, VMError> {
        let imm_size = register_value_from_slice(&self.read_operands(1)?) as usize;
        let x = register_value_from_slice(&self.read_operands(1 + imm_size)?[1..]);
        log_disassembly!("cache ${x} :: DEPRECATED");
        // let fake_sp = self.registers.get_register(STACK_POINTER)?.read();
        // for register_code in 1..=x {
        //     let register_value = self.registers.get_register(register_code as u8)?.read();
        //     self.push(register_value)?;
        // }
        // let real_sp = self.registers.get_register(STACK_POINTER)?.read();
        // self.registers
        //     .get_mut_register(FRAME_POINTER)?
        //     .write(real_sp);
        // self.registers
        //     .get_mut_register(STACK_POINTER)?
        //     .write(fake_sp);

        Ok(OPCODE_BYTES + 1 + imm_size)
    }
    pub fn op_restore(&mut self) -> Result<usize, VMError> {
        let x = self.pop()?;
        log_disassembly!("restore (${x}) :: DEPRECATED");
        // for register_code in x..=1 {
        //     let restored_value = self.pop()?;
        //     let register = self.registers.get_mut_register(register_code as u8)?;
        //     register.write(restored_value);
        // }
        Ok(OPCODE_BYTES) // does not take arguments
    }

    // dest_vmfd str_ptr str_len
    pub fn op_fopen(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES + REGISTER_BYTES + REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;

        let (str_ptr_name, str_ptr) = self.registers.get_register(operand_bytes[1])?.extract();
        let (str_len_name, str_len) = self.registers.get_register(operand_bytes[2])?.extract();
        let dest_vmfd_reg = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!(
            "fopen {}, {}, {}",
            dest_vmfd_reg.name(),
            str_ptr_name,
            str_len_name
        );
        let file_path = match String::from_utf8(self.memory.read_bytes(str_ptr, str_len as usize)?)
        {
            Ok(s) => s,
            Err(e) => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("vm failed to parse file name :: {e}"),
                ))
            }
        };
        let vmfd = self.vm_host_bridge.fopen(&file_path)?;
        dest_vmfd_reg.write(vmfd as u64);
        Ok(OPCODE_BYTES + bytes_read)
    }
    // vmfd buf_ptr buf_len read_len
    pub fn op_fread(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES + REGISTER_BYTES + REGISTER_BYTES + REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let (vmfd_name, vmfd) = self.registers.get_register(operand_bytes[0])?.extract();
        let (buf_ptr_name, buf_ptr) = self.registers.get_register(operand_bytes[1])?.extract();
        let (buf_len_name, buf_len) = self.registers.get_register(operand_bytes[2])?.extract();
        let (read_len_name, read_len) = self.registers.get_register(operand_bytes[3])?.extract();
        log_disassembly!("fread {vmfd_name}, {buf_ptr_name}, {buf_len_name}, {read_len_name}");
        let bytes = self
            .vm_host_bridge
            .fread(vmfd as usize, read_len as usize)?;
        let bytes_trunicated = if bytes.len() > buf_len as usize {
            &bytes[0..buf_len as usize]
        } else {
            &bytes
        };
        self.memory.write_bytes(buf_ptr, &bytes_trunicated)?;
        Ok(OPCODE_BYTES + bytes_read)
    }
    // vmfd write_buf write_len
    pub fn op_fwrite(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES + REGISTER_BYTES + REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let (vmfd_name, vmfd) = self.registers.get_register(operand_bytes[0])?.extract();
        let (write_buf_name, write_buf) = self.registers.get_register(operand_bytes[1])?.extract();
        let (write_len_name, write_len) = self.registers.get_register(operand_bytes[2])?.extract();
        log_disassembly!("fwrite {vmfd_name}, {write_buf_name}, {write_len_name}");
        let bytes = self.memory.read_bytes(write_buf, write_len as usize)?;
        self.vm_host_bridge.fwrite(vmfd as usize, &bytes)?;
        Ok(OPCODE_BYTES + bytes_read)
    }
    // vmfd seek_bytes direction 0|1
    pub fn op_fseek(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES + REGISTER_BYTES + REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let (vmfd_name, vmfd) = self.registers.get_register(operand_bytes[0])?.extract();

        let (seek_bytes_name, seek_bytes) =
            self.registers.get_register(operand_bytes[1])?.extract();

        let (direction_name, direction) = self.registers.get_register(operand_bytes[2])?.extract();

        self.vm_host_bridge
            .fseek(vmfd as usize, seek_bytes as usize, direction as u8)?;
        log_disassembly!("fseek {vmfd_name}, {seek_bytes_name}, {direction_name}");
        Ok(OPCODE_BYTES + bytes_read)
    }

    pub fn op_ftell(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES + REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let (vmfd_name, vmfd) = self.registers.get_register(operand_bytes[1])?.extract();
        let dest_reg = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!("ftell {}, {vmfd_name}", dest_reg.name());
        let position = self.vm_host_bridge.ftell(vmfd as usize)?;
        dest_reg.write(position as u64);
        Ok(OPCODE_BYTES + bytes_read)
    }
    // vmfd
    pub fn op_fclose(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let (vmfd_name, vmfd) = self.registers.get_register(operand_bytes[0])?.extract();

        log_disassembly!("fclose {vmfd_name}");
        self.vm_host_bridge.fclose(vmfd as usize)?;
        Ok(OPCODE_BYTES + bytes_read)
    }

    // wrapper for debug shell or something idk
    pub fn op_breakpoint(&mut self) -> Result<usize, VMError> {
        if !self.ignore_breakpoints {
            log_disassembly!("breakpoint");
            match self.debug_shell() {
                Ok(()) => (),
                Err(err) => match err.code {
                    VMErrorCode::ShellExit => (),
                    _ => return Err(err),
                },
            };
            self.ignore_breakpoints = false;
        } else {
            log_disassembly!("breakpoint ignored");
        }

        Ok(OPCODE_BYTES)
    }
}
