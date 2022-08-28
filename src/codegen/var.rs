

/// variable type
#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    I32,
    I64,
    Struct,
    Array,
    Pointer,
    Unknown
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarValue {
    Int(usize),
    Array{
        bits: usize, 
        elements: Vec<usize>
    },
    Pointer(String)
}

impl Ty {
    pub fn size(&self) -> usize {
        match &self {
            Ty::I32 => { 4 },
            Ty::I64 => { 8 },
            _ => { 0 }
        }
    }
}

/// Variable
#[derive(Debug)]
pub struct Var {
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
    pub(crate) name: String,
    /// is constant
    pub(crate) is_constant: bool,
    // Thread Local
    pub(crate) is_tls: bool
}

impl Var {
    pub fn uninit() -> Self {
        Self {
            ty: Ty::Unknown,
            global: false,
            is_static: false,
            is_local: false,
            initiazed: false,
            init_data: None,
            size: 0,
            align: 0,
            name: String::new(),
            is_constant: false,
            is_tls: false
        }
    }

}