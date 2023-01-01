use super::{ Program, Op, Function, Result, Error, ConstValue, ProgInner, Var, func };
use llvm_ir::{Name, IntPredicate};
use llvm_ir::instruction::{Xor, Load, Store, Alloca, Add, Sub, Mul, SDiv, ICmp, ZExt};
use llvm_ir::terminator::{Ret, Br, CondBr};
use crate::utils::{ parse_operand, parse_type, parse_operand_2 };
use crate::ir::{VirtualReg, StackVar};

impl Program {
    pub(crate) fn handle_alloca(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Alloca) -> Result<()> {
        let num_elements = &inst.num_elements;
        let allocated_type = &inst.allocated_type;
        let dest = &inst.dest;
        match parse_operand(num_elements) {
            Some(Op::ConstValue(op)) => {
                match op {
                    ConstValue::Num(value, size) => {
                        let size = if size >= 8 {
                            size
                        }else{
                            8
                        };
                        VirtualReg::allocate_virt_stack_var(self, func, size, dest.clone());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        
        // if let Ok((ty, size)) = parse_type(allocated_type) {
        //     let reg = &inst.dest;
        //     let mut func_inner = func.inner.borrow_mut();
        //     if func_inner.locals.iter().position(|local| local.name == Some(reg.clone())).is_none() {
        //         let mut local_var = Var::uninit();
        //         // Set local variable type
        //         local_var.ty = ty;
        //         // Set local variable size
        //         local_var.size = size;
        //         // Set local variable name
        //         local_var.name = Some(reg.clone());
        //         // Set stack variable(address, size)
        //         let stack_var = VirtualReg::allocate_virt_stack_var(self, func, offset);
        //         local_var.local_val = Some(VirtualReg::Stack(stack_var));
        //         func_inner.locals.push(local_var);
        //     } 
        // }else{
        //     // warning!("Fail to parse type: {:?}", &alloca.allocated_type);
        // }
        Ok(())
    }


    /// Handle xori instruction
    /// xori rd, rs1, imm12
    /// Note, XORI rd, rs1, -1 rs1 (assembler pseudoinstruction NOT rd, rs ).
    pub(crate) fn handle_xor(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Xor) -> Result<()> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match (parse_operand(op0), parse_operand(op1)) {
            (Some(Op::LocalValue(op1)), Some(Op::ConstValue(op2))) => {
                // op is local variable, op2 is immediate
                let ConstValue::Num(imm, _) = op2;
                let local_var = func.find_local_var(op1).ok_or(Error::new("Fail to find local variable"))?;
                let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to load reg var"))?;
                match &local_var {
                    VirtualReg::Stack(rs1_stack_var) => {
                        // stack_var
                        let rs1_reg_var = rs1_stack_var.load_stack_var(self, prog_inner).ok_or(Error::new("Fail to load stack var"))?;
                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                        self.write_asm(asm);
                    },
                    VirtualReg::Reg(rs1_reg_var) => {
                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                        self.write_asm(asm);
                    }
                }
                Ok(())
            },
            _ => { Err(Error::new("Invalid xor instruction")) }
        }
    }

    /// handle store instruction
    pub(crate) fn handle_store(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Store) -> Result<()> {
        let address = &inst.address;
        let value = &inst.value;
        match (parse_operand(address), parse_operand(value)) {
            (Some(Op::LocalValue(name)), Some(Op::ConstValue(constval))) => {
                let local = func.find_local_var(name).ok_or(Error::new("Fail to find local var"))?;
                match &local {
                    VirtualReg::Stack(stack_var) => {
                        let addr = stack_var.addr;
                        match constval {
                            ConstValue::Num(val, _) => {
                                let name = Name::Name(Box::new(String::from("temp")));
                                let temp_reg;
                                if !func.local_var_exist(name.clone()) {
                                    temp_reg = VirtualReg::allocate_virt_reg_var(prog_inner, func, name.clone()).ok_or(Error::new("Fail to allocate register"))?;
                                    
                                }else{
                                    let local_var = func.find_local_var(name).ok_or(Error::new("Fail to find reg"))?;
                                    match local_var {
                                        VirtualReg::Reg(reg) => { temp_reg = reg },
                                        VirtualReg::Stack(stack) => { todo!() }
                                    }
                                }
                                let asm = format!("    addi {}, zero, {}", temp_reg.name, val);
                                self.write_asm(asm);
                                let asm = format!("    sd {}, -{}(fp)", temp_reg.name, addr);
                                self.write_asm(asm)
                            }
                        }
                    }
                    VirtualReg::Reg(reg) => {

                    }
                }
            }
            _ =>{ return Err(Error::new("Fail to parse operand"))}
        }
        Ok(())
    }

    /// handle load instruction
    pub(crate) fn handle_load(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Load) -> Result<()> {
        let address = &inst.address;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand(address) {
            Some(Op::LocalValue(op)) => {
                let local_virt_reg = func.find_local_var(op).ok_or(Error::new("Fail to find local var"))?;
                match local_virt_reg {
                    VirtualReg::Stack(stack_var) => {
                        let offset = stack_var.addr;
                        let asm = format!("    ld {}, -{}(fp)", dest_reg_var.name, offset);
                        self.write_asm(asm);
                    },
                    _ => { return Err(Error::new("Fail to find stack var")) }
                }
            },
            _ => {}
        }
        Ok(())
    }

    /// handle add instruction
    pub(crate) fn handle_add(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Add) -> Result<()> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand_2(op0, op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                        let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                        // println!("[Debug] var1: {:?}, var2: {:?}", var1, var2);
                        match (var1, var2) {
                            (VirtualReg::Stack(stack1), VirtualReg::Stack(stack2)) => {
                                todo!();
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                let asm = format!("    addw {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                                self.write_asm(asm);
                            },
                            _ => { todo!() }
                        }

                    },
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    }

     /// handle sub instruction
     pub(crate) fn handle_sub(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Sub) -> Result<()> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand_2(op0, op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                        let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                        // println!("[Debug] var1: {:?}, var2: {:?}", var1, var2);
                        match (var1, var2) {
                            (VirtualReg::Stack(stack1), VirtualReg::Stack(stack2)) => {
                                todo!();
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                let asm = format!("    sub {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                                self.write_asm(asm);
                            },
                            _ => { todo!() }
                        }

                    },
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    }

    /// handle mul instruction
    pub(crate) fn handle_mul(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Mul) -> Result<()> {
    let op0 = &inst.operand0;
    let op1 = &inst.operand1;
    let dest = &inst.dest;
    let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
    match parse_operand_2(op0, op1) {
        Some((ans1, ans2)) => {
            match (ans1, ans2) {
                (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                    let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                    let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                    match (var1, var2) {
                        (VirtualReg::Stack(stack1), VirtualReg::Stack(stack2)) => {
                            todo!();
                        },
                        (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                            let asm = format!("    mul {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                            self.write_asm(asm);
                        },
                        _ => { todo!() }
                    }

                },
                _ => {}
            }
        },
        None => return Err(Error::new("Fail to parse operand"))
    }
    Ok(())
}

    /// handle sdiv instruction
    pub(crate) fn handle_sdiv(&self, prog_inner: &mut ProgInner, func: &Function, inst: &SDiv) -> Result<()> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand_2(op0, op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                        let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                        match (var1, var2) {
                            (VirtualReg::Stack(stack1), VirtualReg::Stack(stack2)) => {
                                todo!();
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                let asm = format!("    div {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                                self.write_asm(asm);
                            },
                            _ => { todo!() }
                        }
    
                    },
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    }

    /// handle icmp instruction
    pub(crate) fn handle_icmp(&self, prog_inner: &mut ProgInner, func: &Function, inst: &ICmp) -> Result<()> {
        let predicate = &inst.predicate;
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand_2(op0, op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                        let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                        match (var1, var2) {
                            (VirtualReg::Stack(stack1), VirtualReg::Stack(stack2)) => {
                                todo!();
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                match predicate {
                                    IntPredicate::EQ => {
                                        let name = Name::Name(Box::new(String::from("temp")));
                                        let temp_reg;
                                        if !func.local_var_exist(name.clone()) {
                                            temp_reg = VirtualReg::allocate_virt_reg_var(prog_inner, func, name.clone()).ok_or(Error::new("Fail to allocate register"))?;
                                            
                                        }else{
                                            let local_var = func.find_local_var(name).ok_or(Error::new("Fail to find reg"))?;
                                            match local_var {
                                                VirtualReg::Reg(reg) => { temp_reg = reg },
                                                VirtualReg::Stack(stack) => { todo!() }
                                            }
                                        }
                                        let asm = format!("\txor {}, {}, {}", temp_reg.name, reg1.name, reg2.name);
                                        self.write_asm(asm);
                                        let asm = format!("\tseqz {}, {}", dest_reg_var.name, temp_reg.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::NE => {

                                    },
                                    IntPredicate::SGE => {

                                    },
                                    IntPredicate::SGT => {

                                    },
                                    IntPredicate::SLT => {

                                    },
                                    IntPredicate::UGE => {

                                    },
                                    IntPredicate::UGT => {

                                    },
                                    IntPredicate::ULE => {

                                    },
                                    IntPredicate::ULT => {

                                    },
                                    _ => {}
                                }
                            },
                            _ => { todo!() }
                        }
    
                    },
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    } 

    /// handle zext
    pub(crate) fn handle_zext(&self, prog_inner: &mut ProgInner, func: &Function, inst: &ZExt) -> Result<()> {
        let dest = &inst.dest;
        let op = &inst.operand;
        if let Some(op) = parse_operand(op) {
            match op {
                Op::LocalValue(name) => {
                    let var = func.find_local_var(name.clone()).ok_or(Error::new("Fail to find var"))?;
                    match var {
                        VirtualReg::Reg(reg) => {
                            func.remove_local_var(name.clone());
                            VirtualReg::insert_virt_reg_var(prog_inner, func, dest.clone(), reg);
                        },
                        VirtualReg::Stack(stack) => {
                            todo!()
                        }
                    }
                }
                Op::ConstValue(num) => { panic!() }
            }
        }else{
            panic!()
        }
        Ok(())
    }


    /// handle ret instruction
    pub(crate) fn handle_ret(&self, func: &Function, inst: &Ret) -> Result<()> {
        // return 
        if let Some(op) = &inst.return_operand {
            match parse_operand(op) {
                Some(Op::ConstValue(constval)) => {
                    match constval {
                        ConstValue::Num(num, _) => {
                            let asm = format!("    li a0, {}", num);
                            self.write_asm(asm);
                        }
                    }
                }
                Some(Op::LocalValue(name)) => {
                    match func.find_local_var(name) {
                        Some(VirtualReg::Reg(reg_var)) => {
                            let asm = format!("    mv a0, {}", reg_var.name);
                            self.write_asm(asm);
                        },
                        Some(VirtualReg::Stack(stack_var)) => {
                            todo!()
                        },
                        _ => { return Err(Error::new("Fail to find local var")) }
                    }
                }
                _ => { return Err(Error::new("Fail to parse operand")) }
            }
        }
        self.write_asm("    # function return");
        self.write_asm("    mv sp, fp");
        self.write_asm("    ld fp, 0(sp)");
        self.write_asm("    ld ra, 8(sp)");
        self.write_asm("    addi sp, sp, 16");
        self.write_asm("    ret\n\n");
        Ok(())
    }

    /// handle br
    pub(crate) fn handle_br(&self, func: &Function, inst: &Br) -> Result<()> {
        let llvm_name = inst.dest.clone();
        if let Some(label) = func.find_label(llvm_name.clone()) {
            let asm = format!("j {}", label.label_name);
            self.write_asm(asm);
            return Ok(())
        }
        Err(Error::LabelNotFoundErr{ err: format!("Fail to found label {}", llvm_name)})
    }

    pub(crate) fn handle_condbr(&self, prog_inner: &mut ProgInner, func: &Function, inst: &CondBr) -> Result<()> {
        let condvar = &inst.condition;
        let true_dest = inst.true_dest.clone();
        let false_dest = inst.false_dest.clone();
        let true_dest = func.find_label(true_dest.clone()).ok_or(Error::LabelNotFoundErr{ err: format!("Fail to find label {:?}", true_dest.clone())})?;
        let false_dest = func.find_label(false_dest.clone()).ok_or(Error::LabelNotFoundErr{ err: format!("Fail to find label {:?}", false_dest.clone())})?;
        if let Some(condvar) = parse_operand(condvar) {
            match condvar {
                Op::LocalValue(llvm_reg) => {
                    let phy_reg = func.find_local_var(llvm_reg.clone()).ok_or(Error::RegNotFoundErr{ err: format!("Fail to find reg {}", llvm_reg)})?;
                    if let VirtualReg::Reg(reg) = phy_reg {
                        let asm = format!("\tbne {}, zero, {}", reg.name, true_dest.label_name);
                        self.write_asm(asm);
                        let asm = format!("\tj {}", false_dest.label_name);
                        self.write_asm(asm);
                        return Ok(())
                    }else{
                        return Err(Error::new(format!("Unexpected variable type, {:?}", llvm_reg)));
                    }
                },
                _ => { return Err(Error::new(format!("Unexpected local variable type, {:?}", condvar)));}
            }
        }
        Err(Error::ParseErr{ err: format!("Fail to found parse {:?}", condvar)})
    }
}