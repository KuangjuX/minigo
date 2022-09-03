use std::{fs::File, io::Write as Write2};
use lazy_static::lazy_static;
use std::fmt::{self, Write };
use std::sync::Mutex;

// pub static mut LOG: Log = Log::new("log.txt");
lazy_static!{
    pub static ref LOG: Mutex<Log> = Mutex::new(Log::new("log.txt"));
}

impl Write for Log {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.file.write(s.as_bytes()).unwrap();
        Ok(())
    }
}


pub struct Log {
    file: File
}

impl Log {
    pub fn new<S>(log: S) -> Self where S: Into<String> {
        let file = File::create(log.into()).unwrap();
        Self{
            file
        }
    }

    // pub fn write(&mut self, args: fmt::Arguments) {
    //     self.write_fmt(args).unwrap();
    // }

    pub fn debug<S>(&mut self, info: S) where S: Into<String> {
        let info = format!("\n[Debug] {}\n", info.into());
        self.file.write(info.as_bytes()).unwrap();
    }
}

pub fn log_write(args: fmt::Arguments) {
    let mut log = LOG.lock().unwrap();
    log.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::log::log_write(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

