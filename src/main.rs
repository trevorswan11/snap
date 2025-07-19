#![allow(dead_code)]
use std::error::Error;

mod cli;
mod img;

fn main() -> Result<(), Box<dyn Error>> {
    cli::dispatcher::run()
}
