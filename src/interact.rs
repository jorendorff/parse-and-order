use std::io::{self, Write};
use std::str::FromStr;

use crate::error::MyError;

pub fn read_line(prompt: &str) -> Result<String, MyError> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer)?;
    if buffer.is_empty() {
        // User typed the EOF character
        return Err(MyError::Quit);
    }
    if buffer.ends_with('\n') {
        buffer.pop();
    }
    if buffer == ":quit" || buffer == ":q" {
        return Err(MyError::Quit);
    }
    Ok(buffer)
}

pub fn read<T>(prompt: &str) -> Result<T, MyError>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    loop {
        let line = read_line(prompt)?;
        match line.parse() {
            Ok(value) => return Ok(value),
            Err(err) => println!("{}", err),
        }
    }
}

pub fn confirm(prompt: &str) -> Result<bool, MyError> {
    loop {
        let line = read_line(prompt)?;
        if line == "y" {
            return Ok(true);
        } else if line == "n" {
            return Ok(false);
        } else {
            println!("Please enter y or n (or :q to quit)");
        }
    }
}
