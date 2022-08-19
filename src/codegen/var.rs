/// variable type
#[derive(Debug)]
pub enum Ty {
    I32,
    I64,
    Struct,
    Array,
    Unknown
}

#[derive(Debug)]
pub struct Var {
    /// variable type
    pub(crate) ty: Ty,
    /// global variable
    pub(crate) global: bool,
    /// init
    pub(crate) init: Option<usize>,
    /// variable size
    pub(crate) size: usize,
    /// variable align
    pub(crate) align: usize,
    /// variable name
    pub(crate) name: String,
}

impl Var {
    pub fn uninit() -> Self {
        Self {
            ty: Ty::Unknown,
            global: false,
            init: None,
            size: 0,
            align: 0,
            name: String::new()
        }
    }

    pub fn new(
        ty: Ty, global: bool, init: Option<usize>, 
        size: usize, align: usize, name: String
    ) -> Self {
        Self {
            ty,
            global,
            init,
            size,
            align, 
            name
        }
    }
}