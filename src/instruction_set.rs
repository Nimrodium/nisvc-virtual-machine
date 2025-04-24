use crate::{
    cpu::{DecodedInstruction, CPU},
    log_disassembly, ExecutionError,
};

impl CPU {
    fn op_add(&mut self, decoded: DecodedInstruction) -> Result<(), ExecutionError> {
        let (dest_code, op1, op2) = Self::decode_trinary_register_operation(decoded);
        let sum = op1.read() + op2.read();
        let dest = self.registers.get_mut_register(dest_code)?;
        dest.write(sum);
        log_disassembly!("add {}, {}, {}", dest.name(), op1.name(), op2.name());
        Ok(())
    }
}
