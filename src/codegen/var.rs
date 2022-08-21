

/// variable type
#[derive(Debug)]
pub enum Ty {
    I32,
    I64,
    Struct,
    Array,
    Unknown
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
    /// init
    pub(crate) initiazed: bool,
    /// variable size
    pub(crate) size: usize,
    /// variable align
    pub(crate) align: usize,
    /// variable name
    pub(crate) name: String,
    /// is constant
    pub(crate) is_constant: bool
}

impl Var {
    pub fn uninit() -> Self {
        Self {
            ty: Ty::Unknown,
            global: false,
            is_static: false,
            initiazed: false,
            size: 0,
            align: 0,
            name: String::new(),
            is_constant: false
        }
    }

}