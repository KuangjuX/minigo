
use std::fs::File;
use llvm_ir::module::{Linkage, GlobalVariable};
use llvm_ir::constant::Constant;
use crate::codegen::{Program, Function, VarType};
use crate::codegen::{ Ty, Var, VarValue };
use crate::debug;
use llvm_ir::{Module, Type, self};



mod virtual_reg;

pub use virtual_reg::{VirtualReg, StackVar, RegVar};


pub struct IR {
    pub(crate) module: Module,
}

impl IR {
    pub fn new<S>(bc_file: S) -> Self 
    where S: Into<String> 
    {
        let module = Module::from_bc_path(bc_file.into()).unwrap();
        Self {
            module
        }
    }

    fn debug_function(&self, function: &llvm_ir::Function) {
        debug!("name: {}", function.name);
        debug!("parameters: {:?}", function.parameters);
        debug!("return_type: {:?}", function.return_type);
        debug!("basic_blocks: ");
        for block in function.basic_blocks.iter() {
            debug!("block name: {:?}", block.name);
            for inst in block.instrs.iter() {
                debug!("inst: {:?}", inst);
            }
            debug!("terminator: {:?}", block.term);
        }
    }

    fn parse_init_value(&self, var: &GlobalVariable) -> Option<VarValue> {
        if let Some(constref) = &var.initializer {
            let initval = &**constref;
            match initval {
                Constant::Int{bits, value} => {
                    Some(VarValue::Num(*value as usize, (*bits / 8) as usize))
                },

                Constant::GetElementPtr(element_ptr) => {
                    let addr = &*element_ptr.address;
                    if let Constant::GlobalReference{name, ty} = addr {
                        let mut name = format!("{}", *name);
                        if name.chars().nth(0) == Some('%') {
                            name.remove(0);
                        }
                        return Some(VarValue::Pointer(name))
                    }
                    None
                },

                Constant::Array{element_type, elements} => {
                    let element_type = &**element_type;
                    let mut array: Vec<usize> = vec![];
                    match element_type {
                        Type::IntegerType { bits } => {
                            for element in elements {
                                let element = &**element;
                                match element {
                                    &Constant::Int{bits, value} => {
                                        array.push(value as usize);
                                    },
                                    _ => {}
                                }
                                
                            }
                            return Some(VarValue::Array{
                                bits: *bits as usize, 
                                elements: array
                            })
                        },
                        _ => { None }
                    }
                }

                _ => {
                    None
                }
            }
        }else{
            None
        }
    }

    /// parse variable type && size
    fn parse_variable_type(&self, var: &GlobalVariable, new_var: &mut Var) {
        let ty = &*var.ty;
        match ty {
            Type::PointerType{ pointee_type, addr_space } => {
                let pointee_ty = &**pointee_type;
                match pointee_ty {
                    Type::IntegerType{ bits } => {
                        let init_val = self.parse_init_value(var);
                        if let Some(val) = init_val {
                            match val {
                                VarValue::Num(_, size) => {
                                    new_var.ty = Ty::Num;
                                    new_var.size = size;
                                    new_var.init_data = Some(val);
                                },
                                _ => {}
                            }
                        }
                        
                    },

                    Type::ArrayType{element_type, num_elements} => {
                        let element_type = &**element_type;
                        match element_type {
                            &Type::IntegerType{ bits} => {
                                if let Some(val) = self.parse_init_value(var) {
                                    new_var.ty = Ty::Array;
                                    if let VarValue::Array{ bits, elements} = &val {
                                        new_var.size = (bits/8) * elements.len();
                                    }
                                    new_var.init_data = Some(val.clone());
                                }
                            },
                            _ => {}
                        }
                    }

                    Type::PointerType{pointee_type, addr_space} => {
                        let init_val = self.parse_init_value(var);
                        new_var.ty = Ty::Pointer;
                        new_var.size = 8;
                        new_var.init_data = init_val;
                    }   
                    _ => {}
                }
            }

            _ => {}
        }
    }

    /// parse global variable
    fn parse_variable(&self, var: &GlobalVariable) -> Var {
        let mut new_var = Var::uninit();
        new_var.global = true;
        // check link option
        match var.linkage {
            Linkage::Internal => {
                new_var.is_static = true;
            },

            Linkage::External => {
                new_var.is_static = false;
            },

            _ => {}
        }
        // check init
        match &var.initializer {
            Some(_) => {
                new_var.initiazed = true;
            },

            None => {
                new_var.initiazed = false;
            }
        }

        new_var.is_constant = var.is_constant;
        self.parse_variable_type(var, &mut new_var);
        new_var.align = var.alignment as usize;
        new_var.name = Some(var.name.clone());
        new_var
    }


    /// parse IR function
    fn parse_function(&self, llvm_func: &llvm_ir::Function) -> Function {
        self.debug_function(llvm_func);
        let mut func = Function::uninit();
        func.name = llvm_func.name.clone();
        for param in llvm_func.parameters.iter() {
            let ty = &*param.ty.clone();
            match ty {
                &Type::IntegerType{bits} => {
                    // add stack depth
                    // func.push_var(8);
                    // initiaize param 
                    let mut param_var = Var::uninit();
                    param_var.var_type = VarType::Param;
                    param_var.ty = Ty::Num;
                    param_var.size = if bits / 8 <= 8 {
                        8
                    }else{ bits as usize / 8 };
                    param_var.name = Some(param.name.clone());
                    func.params.push(param_var);
                },
                _ => {}
            }
        }
        for block in llvm_func.basic_blocks.iter() {
            func.blocks.push_back(block.clone());
        }
        func
    }

    pub fn parse(&self, asm: &str) -> Program {
        let asm = File::create(asm).unwrap();
        let program = Program::new(asm);
        // parse global variable
        let inner = unsafe{ &mut *program.inner.get() };
        for var in self.module.global_vars.iter() {
            let new_var = self.parse_variable(var);
            inner.vars.push_back(new_var);
        }
        for function in self.module.functions.iter() {
            let func = self.parse_function(function);
            inner.funcs.push_back(func);
        }
        drop(inner);
        program
    }
}