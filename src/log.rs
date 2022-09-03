use std::{fs::File, io::Write as Write2};
use std::fmt::{self, Write };
use std::sync::Mutex;
use lazy_static::lazy_static;


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
        $crate::log::log_write(format_args!(concat!("[DEBUG] ", $fmt, "\n") $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::log::log_write(format_args!(concat!("[INFO] ", $fmt, "\n") $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! warning {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::log::log_write(format_args!(concat!("[WARNING] ", $fmt, "\n") $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::log::log_write(format_args!(concat!("[ERROR] ", $fmt, "\n") $(, $($arg)+)?))
    }
}

