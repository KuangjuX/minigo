
// #[derive(Debug)]
// pub struct Error {
//     pub message: String
// }

// impl Error {
//     pub fn new<S>(msg: S)  -> Self where S: Into<String> {
//         Self {
//             message: msg.into()
//         }
//     }
// }

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    RegAllocateErr{ err: String },
    RegNotFoundErr{ err: String },
    LabelNotFoundErr{ err: String },
    ParseErr{ err: String },
    DefaultErr{ err: String }
}

impl Error {
    pub fn new<S>(err: S) -> Error where S: Into<String> {
        Self::DefaultErr{ err: err.into() }
    }
}

