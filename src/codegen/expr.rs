use super::{ Program, Op, Function, Error, ConstValue };
use llvm_ir::instruction::Xor;
use crate::{utils::parse_operand, ir::{StackVar, VirtualReg}};

impl Program {
    /// Handle xori instruction
    /// xori rd, rs1, imm12
    /// Note, XORI rd, rs1, -1 rs1 (assembler pseudoinstruction NOT rd, rs ).
    pub(crate) fn handle_xor(&self, func: &Function, inst: &Xor) -> Result<(), Error> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match (parse_operand(op0), parse_operand(op1)) {
            (Some(Op::LocalValue(op1)), Some(Op::ConstValue(op2))) => {
                // op is local variable, op2 is immediate
                let local_var = func.find_local_var(op1).ok_or(Error::new("Fail to find local variable"))?;
                match &local_var {
                    VirtualReg::Stack(stack_var) => {
                        let rs1_reg_var = stack_var.load_stack_var(self).ok_or(Error::new("Fail to load stack var"))?;
                        let dest_stack_var = VirtualReg::naive_allocate_virt_reg(self, func, 8);
                        let dest_reg_var = dest_stack_var.load_stack_var(self).ok_or(Error::new("Fail to load stack var"))?;
                        let ConstValue::Num(imm, _) = op2;
                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm);
                        self.write_asm(asm);
                        Ok(())
                    },
                    VirtualReg::Reg(reg_var) => {
                        Ok(())
                    }
                } 
                   
                
            },
            _ => { Err(Error::new("Invalid xor instruction")) }
        }
    }
}