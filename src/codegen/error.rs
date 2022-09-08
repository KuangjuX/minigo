
#[derive(Debug)]
pub struct Error {
    pub message: String
}

impl Error {
    pub fn new<S>(msg: S)  -> Self where S: Into<String> {
        Self {
            message: msg.into()
        }
    }
}