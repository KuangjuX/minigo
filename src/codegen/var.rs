use llvm_ir::name::Name;
use crate::ir::VirtualReg;

#[derive(Debug, PartialEq)]
pub enum VarType {
    Global,
    Local,
    Param,
    Uninit
}



/// variable type
#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Num,
    Struct,
    Array,
    Pointer,
    Unknown
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarValue {
    /// Num: (value, size)
    Num(usize, usize),
    Array{
        bits: usize, 
        elements: Vec<usize>
    },
    Pointer(String)
}

impl Ty {
    pub fn size(&self) -> usize {
        match &self {
            Ty::Num => { 8 },
            _ => { 0 }
        }
    }
}

/// Variable
#[derive(Debug)]
pub struct Var {
    pub var_type: VarType,
    /// variable type
    pub(crate) ty: Ty,
    /// global variable
    pub(crate) global: bool,
    /// static global
    pub(crate) is_static: bool,
    /// is local, used in function
    pub(crate) is_local: bool,
    /// init
    pub(crate) initiazed: bool,
    /// init data
    pub(crate) init_data: Option<VarValue>,
    /// variable size
    pub(crate) size: usize,
    /// variable align
    pub(crate) align: usize,
    /// variable name
    pub(crate) name: Option<Name>,
    /// is constant
    pub(crate) is_constant: bool,
    // Thread Local
    pub(crate) is_tls: bool,

    // pub(crate) local_val: Option<VarValue>
    /// Function local variable, represented by virtual reg
    pub(crate) local_val: Option<VirtualReg>
}

impl Var {
    pub fn uninit() -> Self {
        Self {
            var_type: VarType::Uninit,
            ty: Ty::Unknown,
            global: false,
            is_static: false,
            is_local: false,
            initiazed: false,
            init_data: None,
            size: 0,
            align: 0,
            name: None,
            is_constant: false,
            is_tls: false,
            local_val: None
        }
    }

}