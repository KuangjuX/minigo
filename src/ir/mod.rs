use std::fs::File;
use llvm_ir::module::{Linkage, GlobalVariable};
use llvm_ir::name::Name;
use llvm_ir::constant::Constant;
use crate::codegen::Program;
use crate::codegen::{ Ty, Var, VarValue };
use llvm_ir::{Module, Type};

pub struct IR {
    pub(crate) module: Module,
}

impl IR {
    pub fn new<S>(bc_file: S) -> Self 
    where S: Into<String> 
    {
        let module = Module::from_bc_path(bc_file.into()).unwrap();
        // for var in module.global_vars.iter() {
        //     println!("[Debug] {:?}", var);
        // }
        Self {
            module
        }
    }

    fn parse_init_value(&self, var: &GlobalVariable) -> Option<VarValue> {
        // println!("\n\ninitval: {:?}", &var.initializer);
        if let Some(constref) = &var.initializer {
            let initval = &**constref;
            match initval {
                Constant::Int{bits, value} => {
                    Some(VarValue::Int(*value as usize))
                },

                Constant::GetElementPtr(element_ptr) => {
                    println!("\n\n[Debug] ptr var: {:?}", var);
                    let addr = &*element_ptr.address;
                    if let Constant::GlobalReference{name, ty} = addr {
                        let name = format!("{}", *name);
                        return Some(VarValue::Pointer(name))
                    }
                    None
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
            println!("[Debug] var: {:?}", var);
            let ty = &*var.ty;
            match ty {
                Type::PointerType{ pointee_type, addr_space } => {
                    let pointee_ty = &**pointee_type;
                    match pointee_ty {
                        Type::IntegerType{ bits } => {
                            let size = bits / 8;
                            match size  {
                                4 => {
                                    let init_val = self.parse_init_value(var);
                                    if let Some(val) = init_val {
                                        // return (Ty::I32, 4, Some(val))
                                        new_var.ty = Ty::I32;
                                        new_var.size = 4;
                                        new_var.init_data = Some(val);
                                    }
                                },
    
                                8 => {
                                    let init_val = self.parse_init_value(var);
                                    if let Some(val) = init_val {
                                        match val {
                                            VarValue::Int(_) => {
                                                // return (Ty::I64, 8, Some(val))
                                                new_var.ty = Ty::I64;
                                                new_var.size = 8;
                                                new_var.init_data = Some(val);
                                            },
    
                                            // VarValue::Pointer(_) => {
                                            //     // return (Ty::Pointer, 8, Some(val))
                                            //     new_var.ty = Ty::Pointer;
                                            //     new_var.size = 8;
                                            //     new_var.init_data = Some(val);
                                            // },
    
                                            _ => {}
                                        }
                                    }
    
                                },
    
                                _ => {}
                            }
                        },

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
        match &var.name  {
            Name::Name(name) => {
                let name = format!("{}", *name);
                new_var.name = name;
            },

            Name::Number(num) => {}
        }
        new_var
    }

    // fn parse_function(&self) -> Function {

    // }

    pub fn parse(&self) -> Program {
        let asm = File::create("main.S").unwrap();
        let mut program = Program::new(asm);
        // parse global variable
        for var in self.module.global_vars.iter() {
            let new_var = self.parse_variable(var);
            program.vars.push_back(new_var);
        }
        program
    }
}