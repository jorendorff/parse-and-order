use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("quit")]
    Quit,
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("parse error on command line input")]
    Parse(#[from] Box<dyn std::error::Error + 'static>),
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for MyError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        MyError::Custom(format!("{}", msg))
    }
}
