use std::{fs::File, io::Write};
pub struct Log {
    file: File
}

impl Log {
    pub fn new<S>(file: S) -> Self where S: Into<String> {
        let file = File::create("file").unwrap();
        Self{
            file
        }
    }

    pub fn debug<S>(&mut self, info: S) where S: Into<String> {
        let info = format!("\n[Debug] {}\n", info.into());
        self.file.write(info.as_bytes()).unwrap();
    }
}

