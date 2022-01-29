use serde::Deserialize;

use crate::error::MyError;
use crate::model::Zoo;

mod de;
mod error;
mod interact;
mod model;

fn main() {
    println!("The Serde deserialization process is an interaction between two separate,");
    println!("yet equally important, impls:");
    println!("the Deserializer, which parses a data format,");
    println!("and Deserialize, which constructs the Rust values.");
    println!();
    println!("These are their stories.");
    println!();

    let deserializer = de::InteractiveDe::new("zoo");
    match <Zoo as Deserialize>::deserialize(deserializer) {
        Ok(zoo) => {
            println!();
            println!("And here is your finished Zoo:\n{:#?}", zoo)
        }
        Err(MyError::Quit) => {}
        Err(err) => {
            println!("error: {}", err);
            let mut current: &(dyn std::error::Error + 'static) = &err;
            while let Some(cause) = current.source() {
                println!("  caused by: {}", cause);
                current = cause;
            }
        }
    }
    println!("\nExecutive Producer: Jason Orendorff");
}
