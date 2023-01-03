use super::{ Program, Op, Function, Result, Error, ConstValue, ProgInner, InstType};
use llvm_ir::{Name, IntPredicate, Operand};
use llvm_ir::instruction::{Xor, Load, Store, Alloca,  ICmp, ZExt, Call};
use llvm_ir::terminator::{Ret, Br, CondBr};
use crate::utils::{ parse_operand, parse_operand_2};
use crate::ir::{VirtualReg,  RegVar};

impl Program {
    pub(crate) fn handle_alloca(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Alloca) -> Result<()> {
        let num_elements = &inst.num_elements;
        let dest = &inst.dest;
        match parse_operand(num_elements) {
            Some(Op::ConstValue(op)) => {
                match op {
                    ConstValue::Num(_, size) => {
                        // 计算需要分配的栈空间
                        let size = if size >= 8 { size }else{ 8 };
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
                // op1 是变量, op2 是立即数
                match op2 {
                    ConstValue::Num(imm, _) => {
                        let local_var = func.find_local_var(op1).ok_or(Error::new("Fail to find local variable"))?;
                        // 尝试分配一个物理寄存器
                        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
                            Some(dest_reg_var) => {
                                // 分配成功
                                match &local_var {
                                    VirtualReg::Stack(rs1_stack_var) => {
                                        // stack_var
                                        let rs1_reg_var = rs1_stack_var.load_stack_var_1(self, prog_inner);
                                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                    },
                                    VirtualReg::Reg(rs1_reg_var) => {
                                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                    }
                                }
                            },
                            None => {
                                // 分配失败，物理寄存器溢出
                                // 分配一个栈变量
                                let stack_var = VirtualReg::spill_virtual_var(self, func, 8, dest.clone());
                                match &local_var {
                                    VirtualReg::Stack(rs1_stack_var) => {
                                        // stack_var
                                        let rs1_reg_var = rs1_stack_var.load_stack_var_1(self, prog_inner);
                                        // 将栈变量的值写入寄存器中
                                        let asm = format!("\txori {}, {}, {}", rs1_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                        // 将寄存器的值加载到栈变量中
                                        let asm = format!("\tsd {}, -{}(fp)", rs1_reg_var.name, stack_var.addr);
                                        self.write_asm(asm);
                                    },
                                    VirtualReg::Reg(rs1_reg_var) => {
                                        // 获取帮助寄存器
                                        let help_reg = prog_inner.get_help_reg_1();
                                        // 将结果写入帮助寄存器中
                                        let asm = format!("\txori {}, {}, {}", help_reg.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                        // 将帮助寄存器结果写入栈中
                                        let asm = format!("\tsd {}, -{}(fp)", help_reg.name, stack_var.addr);
                                        self.write_asm(asm);
                                    }
                                }
                            }
                        }
                        
                    },
                    _ => { todo!() }
                }
                Ok(())
            },
            _ => { Err(Error::new("Invalid xor instruction")) }
        }
    }

    /// handle store instruction
    /// 将寄存器中的值存入栈中
    pub(crate) fn handle_store(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Store) -> Result<()> {
        let address = &inst.address;
        let value = &inst.value;
        match parse_operand_2(address, value) {
            Some((Op::LocalValue(name), Op::ConstValue(constval))) => {
                let local = func.find_local_var(name).ok_or(Error::new("Fail to find local var"))?;
                match &local {
                    VirtualReg::Stack(stack_var) => {
                        let addr = stack_var.addr;
                        match constval {
                            ConstValue::Num(val, _) => {
                                let help_reg = prog_inner.get_help_reg_1();
                                let asm = format!("    addi {}, zero, {}", help_reg.name, val);
                                self.write_asm(asm);
                                let asm = format!("    sd {}, -{}(fp)", help_reg.name, addr);
                                self.write_asm(asm)
                            },
                            _ => { todo!() }
                        }
                    }
                    VirtualReg::Reg(reg) => {
                        todo!()
                    }
                }
            },
            Some((Op::LocalValue(op1), Op::LocalValue(op2))) => {
                let var1 = func.find_local_var(op1).ok_or(Error::new("Fail to find local var"))?;
                let var2 = func.find_local_var(op2).ok_or(Error::new("Fail to find local var"))?;
                match (var1, var2) {
                    (VirtualReg::Stack(stack), VirtualReg::Reg(reg)) => {
                        let addr = stack.addr;
                        let asm = format!("\tsd {}, -{}(fp)", reg.name, addr);
                        self.write_asm(asm);
                    },
                    (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                        let help_reg_1 = stack_var_2.load_stack_var_1(self, prog_inner);
                        let asm = format!("\tsd {}, -{}(fp)", help_reg_1.name, stack_var_1.addr);
                        self.write_asm(asm);
                    }
                    _ => { todo!() }
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
        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
            Some(dest_reg_var) => {
                // 可以分配寄存器
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
                    _ => { todo!() }
                }
            },
            None => {
                // 物理寄存器分配溢出
                // 分配栈寄存器
                let dest_stack_var = VirtualReg::spill_virtual_var(self, func, 8, dest.clone());
                match parse_operand(address) {
                    Some(Op::LocalValue(op)) => {
                        let local_virt_reg = func.find_local_var(op).ok_or(Error::new("Fail to find local var"))?;
                        let help_reg = prog_inner.get_help_reg_1();
                        match local_virt_reg {
                            VirtualReg::Stack(src_stack_var) => {
                                // 将内存放入帮助寄存器中
                                let offset = src_stack_var.addr;
                                let asm = format!("\tld {}, -{}(fp)", help_reg.name, offset);
                                self.write_asm(asm);
                                // 将帮助寄存器的值放入栈中
                                let asm = format!("\tsd {}, -{}(fp)", help_reg.name, dest_stack_var.addr);
                                self.write_asm(asm);
                            },
                            _ => { return Err(Error::new("Fail to find stack var")) }
                        }
                    },
                    _ => { todo!() }
                }
            }
        }
        Ok(())
    }

    /// FnOnce(op0, op1, dest)
    pub(crate) fn handle_add_sub_mul_div_mod<F>(
        &self, prog_inner: &mut ProgInner, func: &Function, 
        op0: Operand, op1: Operand, dest: Name, f: F) -> Result<()> 
        where F: FnOnce(String, String, String, InstType) -> String
    {
        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
            Some(dest_reg_var) => {
                match parse_operand_2(&op0, &op1) {
                    Some((ans1, ans2)) => {
                        match (ans1, ans2) {
                            (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                                let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                                let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                                match (var1, var2) {
                                    (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                                        // 分配两个帮助寄存器并将两个栈加载到帮助寄存器中
                                        let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                        let help_reg_2 = stack_var_2.load_stack_var_2(self, prog_inner);
                                        // 计算结果并写回目标寄存器
                                        let asm = f(help_reg_1.name, help_reg_2.name, dest_reg_var.name, InstType::R);
                                        self.write_asm(asm);
                                    },
                                    (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                        let asm = f(reg1.name, reg2.name, dest_reg_var.name, InstType::R);
                                        self.write_asm(asm)
                                    },
                                    (VirtualReg::Reg(reg_var_1), VirtualReg::Stack(stack_var_2)) => {
                                        let help_reg_1 = stack_var_2.load_stack_var_1(self, prog_inner);
                                        let asm = f(reg_var_1.name.clone(), help_reg_1.name.clone(), dest_reg_var.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                    },
                                    (VirtualReg::Stack(stack_var_1), VirtualReg::Reg(reg_var_2)) => {
                                        let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                        let asm = f(help_reg_1.name.clone(), reg_var_2.name.clone(), dest_reg_var.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                    }
                                }
        
                            },
                            (Op::LocalValue(var), Op::ConstValue(val)) => {
                                let var = func.find_local_var(var).ok_or(Error::new("Fail to find var"))?;
                                if let ConstValue::Num(num, _) = val {
                                    match var {
                                        VirtualReg::Reg(reg) => {
                                            let asm = f(reg.name, format!("{}", num), dest_reg_var.name, InstType::I);
                                            self.write_asm(asm);
                                        },
                                        VirtualReg::Stack(stack_var) => {
                                            let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                                            let asm = f(help_reg.name, format!("{}", num), dest_reg_var.name, InstType::I);
                                            self.write_asm(asm);
                                        }
                                    }
                                }else{
                                    todo!()
                                }
                                
                            }   
                            _ => {}
                        }
                    },
                    None => return Err(Error::new("Fail to parse operand"))
                }
            }
            None => {
                // 物理寄存器分配溢出
                // 分配栈变量
                let stack_dest_var = VirtualReg::spill_virtual_var(self, func, 8, dest);
                match parse_operand_2(&op0, &op1) {
                    Some((ans1, ans2)) => {
                        match (ans1, ans2) {
                            (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                                let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                                let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                                match (var1, var2) {
                                    (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                                        // 分配两个帮助寄存器并将两个栈加载到帮助寄存器中
                                        let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                        let help_reg_2 = stack_var_2.load_stack_var_2(self, prog_inner);
                                        // 计算结果并写回帮助寄存器
                                        let asm = f(help_reg_1.name.clone(), help_reg_2.name, help_reg_1.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                        // 将帮助寄存器的值存储回栈变量
                                        stack_dest_var.store_stack_var(self, help_reg_1);
                                    },
                                    (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                        let help_reg_1: RegVar = prog_inner.get_help_reg_1().into();
                                        let asm = f(reg1.name, reg2.name, help_reg_1.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                        // 将帮助寄存器的值存储回栈变量
                                        stack_dest_var.store_stack_var(self, help_reg_1);
                                    },
                                    (VirtualReg::Reg(reg_var_1), VirtualReg::Stack(stack_var_2)) => {
                                        let help_reg_1 = stack_var_2.load_stack_var_1(self, prog_inner);
                                        let help_reg_2: RegVar = prog_inner.get_help_reg_2().into();
                                        let asm = f(reg_var_1.name.clone(), help_reg_1.name.clone(), help_reg_2.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                        stack_dest_var.store_stack_var(self, help_reg_2);
                                    },
                                    (VirtualReg::Stack(stack_var_1), VirtualReg::Reg(reg_var_2)) => {
                                        let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                        let help_reg_2: RegVar = prog_inner.get_help_reg_2().into();
                                        let asm = f(help_reg_1.name.clone(), reg_var_2.name.clone(), help_reg_2.name.clone(), InstType::R);
                                        self.write_asm(asm);
                                        stack_dest_var.store_stack_var(self, help_reg_2);
                                    }
                                }
        
                            },
                            (Op::LocalValue(var), Op::ConstValue(val)) => {
                                let var = func.find_local_var(var).ok_or(Error::new("Fail to find var"))?;
                                if let ConstValue::Num(num, _) = val {
                                    match var {
                                        VirtualReg::Reg(reg) => {
                                            let help_reg_1: RegVar = prog_inner.get_help_reg_1().into();
                                            let asm = f(reg.name, format!("{}", num), help_reg_1.name.clone(), InstType::I);
                                            self.write_asm(asm);
                                            // 将帮助寄存器的值存储回栈变量
                                            stack_dest_var.store_stack_var(self, help_reg_1);
                                        },
                                        VirtualReg::Stack(stack_var) => {
                                            let help_reg_1 = stack_var.load_stack_var_1(self, prog_inner);
                                            let help_reg_2:RegVar = prog_inner.get_help_reg_2().into();
                                            let asm = f(help_reg_1.name, format!("{}", num), help_reg_2.name.clone(), InstType::I);
                                            self.write_asm(asm);
                                            // 将帮助寄存器的值存储回栈变量
                                            stack_dest_var.store_stack_var(self, help_reg_2);
                                        }
                                    }
                                }else{
                                    todo!()
                                }
                                
                            }   
                            _ => {}
                        }
                    },
                    None => return Err(Error::new("Fail to parse operand"))
                }
            }
        }
        Ok(())
    }


    /// handle icmp instruction
    pub(crate) fn handle_icmp(&self, prog_inner: &mut ProgInner, func: &Function, inst: &ICmp) -> Result<()> {
        let predicate = &inst.predicate;
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
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
                                        let help_reg = prog_inner.get_help_reg_1();
                                        let asm = format!("\txor {}, {}, {}", help_reg.name, reg1.name, reg2.name);
                                        self.write_asm(asm);
                                        let asm = format!("\tseqz {}, {}", dest_reg_var.name, help_reg.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::NE => {
                                        let help_reg = prog_inner.get_help_reg_1();
                                        let asm = format!("\txor {}, {}, {}", help_reg.name, reg1.name, reg2.name);
                                        self.write_asm(asm);
                                        let asm = format!("\tsnez {}, {}", dest_reg_var.name, help_reg.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::SGT => {
                                        let asm = format!("\tslt {}, {}, {}", dest_reg_var.name, reg2.name, reg1.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::SGE => {
                                        let asm = format!("\tslt {}, {}, {}", dest_reg_var.name, reg2.name, reg1.name);
                                        self.write_asm(asm);
                                        let asm = format!("\txori {}, {}, 1", dest_reg_var.name, dest_reg_var.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::SLT => {
                                        let asm = format!("\tslt {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                                        self.write_asm(asm);
                                    },
                                    IntPredicate::SLE => {
                                        let asm = format!("\tslt {}, {}, {}", dest_reg_var.name, reg1.name, reg2.name);
                                        self.write_asm(asm);
                                        let asm = format!("\txori {}, {}, 1", dest_reg_var.name, dest_reg_var.name);
                                        self.write_asm(asm);
                                    }
                                    _ => { todo!() }
                                }
                            },
                            _ => { todo!() }
                        }
    
                    },
                    (Op::LocalValue(var), Op::ConstValue(val)) => {
                        let var = func.find_local_var(var).ok_or(Error::new("Fail to find var"))?;
                        match var {
                            VirtualReg::Reg(reg) => {
                                if let ConstValue::Num(num, _) = val {
                                    match predicate {
                                        IntPredicate::EQ => {
                                            let help_reg = prog_inner.get_help_reg_1();
                                            let asm = format!("\txori {}, {}, {}", help_reg.name, reg.name, num);
                                            self.write_asm(asm);
                                            let asm = format!("\tseqz {}, {}", dest_reg_var.name, help_reg.name);
                                            self.write_asm(asm);
                                        },
                                        IntPredicate::NE => {
                                            let help_reg = prog_inner.get_help_reg_1();
                                            let asm = format!("\txori {}, {}, {}", help_reg.name, reg.name, num);
                                            self.write_asm(asm);
                                            let asm = format!("\tsnez {}, {}", dest_reg_var.name, help_reg.name);
                                            self.write_asm(asm);
                                        }
                                        IntPredicate::SGT => {
                                            let asm = format!("\tslti {}, {}, {}", dest_reg_var.name, num, reg.name);
                                            self.write_asm(asm);
                                        }
                                        IntPredicate::SGE => {
                                            let asm = format!("\tslti {}, {}, {}", dest_reg_var.name, num, reg.name);
                                            self.write_asm(asm);
                                            let asm = format!("\txori {}, {}, 1", dest_reg_var.name, dest_reg_var.name);
                                            self.write_asm(asm);
                                        }
                                        IntPredicate::SLT => {
                                            let asm = format!("\tslti {}, {}, {}", dest_reg_var.name, reg.name, num);
                                            self.write_asm(asm);
                                        },
                                        IntPredicate::SLE => {
                                            let asm = format!("\tslti {}, {}, {}", dest_reg_var.name, reg.name, num);
                                            self.write_asm(asm);
                                            let asm = format!("\txori {}, {}, 1", dest_reg_var.name, dest_reg_var.name);
                                            self.write_asm(asm);
                                        }
                                        _ => { todo!() }
                                    }
                                }else{
                                    todo!()
                                }
                                
                            },
                            VirtualReg::Stack(stack) => {
                                todo!()
                            }
                        }
                    }
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
    pub(crate) fn handle_ret(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Ret) -> Result<()> {
        // return 
        if let Some(op) = &inst.return_operand {
            match parse_operand(op) {
                Some(Op::ConstValue(constval)) => {
                    match constval {
                        ConstValue::Num(num, _) => {
                            let asm = format!("\tli a0, {}", num);
                            self.write_asm(asm);
                        }
                        _ => { todo!() }
                    }
                }
                Some(Op::LocalValue(name)) => {
                    match func.find_local_var(name) {
                        Some(VirtualReg::Reg(reg_var)) => {
                            let asm = format!("\tmv a0, {}", reg_var.name);
                            self.write_asm(asm);
                        },
                        Some(VirtualReg::Stack(stack_var)) => {
                            let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                            let asm = format!("\tmv a0, {}", help_reg.name);
                            self.write_asm(asm);
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
            let asm = format!("\tj {}", label.label_name);
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

    pub(crate) fn handle_call(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Call) -> Result<()> {
        // 准备参数，完成传参
        // 保存 caller-saved 寄存器
        // 执行汇编中的函数调用指令，开始执行子函数直至其返回
        // 恢复 caller-saved 寄存器
        // 拿到函数调用的返回值，作为函数调用表达式的值

        // STEP1: 首先，保存 caller-saved 寄存器
        // 计算 caller-saved 寄存器所占的栈空间
        // let space: isize = 8 * CALLER_SAVED_REGS.len() as isize;
        // 将栈顶指针下移
        // let asm = format!("\taddi sp, sp, -{}", space);
        // self.write_asm(asm);
        // // 保存寄存器
        let mut index = 0;
        // for reg in CALLER_SAVED_REGS {
        //     let asm = format!("\tsd {}, {}(sp)", reg, index);
        //     self.write_asm(asm);
        //     index += 8;
        // }
        // // 修改函数栈空间大小
        // func.add_stack_size(space as isize);

        // STEP2: 将参数放在 a0 - a7 寄存器中，如果还有其他参数，则以从右向左的顺序压栈
        // 第 9 个参数在栈顶位置
        index = 0;
        let len = inst.arguments.len();
        if len <= 7 {
            for (arg, _) in inst.arguments.iter() {
                if let Some(op) = parse_operand(arg) {
                    match op {
                        Op::LocalValue(name) => {
                            let var = func.find_local_var(name).ok_or(Error::new("Failed to find local var"))?;
                            match var {
                                VirtualReg::Stack(stack_var) => {
                                    todo!()
                                },
                                VirtualReg::Reg(reg_var) => {
                                    let asm = format!("\tmv a{}, {}", index, reg_var.name);
                                    self.write_asm(asm);
                                }
                            }
                        },
                        Op::ConstValue(val) => {
                            if let ConstValue::Num(num, _) = val {
                                let asm = format!("\taddi a{}, zero, {}", index, num);
                                self.write_asm(asm);
                            }else{
                                todo!()
                            }
                            
                        }
                    }
                }
                index += 1;
            }
        }else{
            // 将所有参数全部放在栈里
            // 扩展栈空间
            let size = 8 * len as isize;
            let asm = format!("addi sp, sp, -{}", size);
            self.write_asm(asm);
            func.add_stack_size(size);
            for (arg, _) in inst.arguments.iter() {
                if let Some(op) = parse_operand(arg) {
                    match op {
                        Op::LocalValue(name) => {
                            todo!()
                        },
                        Op::ConstValue(val) => {
                            // 目前只考虑参数是数字的情况
                            match val {
                                ConstValue::Num(num,_) => {
                                    let asm = format!("sd {}, {}(sp)", num, index * 8);
                                    self.write_asm(asm);
                                },
                                _ => { todo!() }
                            }
                            
                        }
                    }
                }
                index += 1;
            }
        }


        // STEP3: 调用 call 指令
        let dest = inst.dest.clone().ok_or(Error::new("[Call] Fail to get target register"))?;
        // 分配物理寄存器
        let dest_reg_var = VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        let func_op = inst.function.clone().right().unwrap();
        // 保存使用过的寄存器
        func.store_regs(self);
        if let Some(func) = parse_operand(&func_op) {
            match func {
                Op::ConstValue(val) => {
                    match val {
                        ConstValue::Ref(symbol) => {
                            match symbol {
                                Name::Name(symbol) => {
                                    let asm = format!("\tcall {}", symbol);
                                    self.write_asm(asm);
                                },
                                Name::Number(num) => {
                                    let asm = format!("\tcall {}", num);
                                    self.write_asm(asm);
                                }
                            }
                        },
                        _ => { return Err(Error::new(format!("[Call] Unexpected function type: {:?}",val )))}
                    }
                  
                },
                _ => { panic!() }
            }
        }else{
            return Err(Error::ParseErr{ err: format!("[Call] Fail to parse function {:?}", func_op)})
        }
        // 恢复使用过的寄存器
        func.restore_regs(self);
        // 获取函数返回值
        let asm = format!("\tmv {}, a0", dest_reg_var.name);
        self.write_asm(asm);
        // 将调用的返回值赋给虚拟寄存器
        // STEP4: 恢复 caller-saved 寄存器
        // index = 0;
        // for reg in CALLER_SAVED_REGS {
        //     let asm = format!("\tld {}, {}(sp)", reg, index);
        //     self.write_asm(asm);
        //     index += 8;
        // }
        // // 修改栈顶指针
        // let asm = format!("\taddi sp, sp, {}", space);
        // self.write_asm(asm);
        // func.add_stack_size(-space as isize);

        Ok(())
    }
}