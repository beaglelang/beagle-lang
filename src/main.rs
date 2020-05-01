// #![feature(async_closure)]

use frontend::Driver;

use std::path::Path;
use std::thread;

fn main() -> std::io::Result<()> {
    let driver = Driver::new();
    let tir = futures::executor::block_on(driver.parse_module("test.txt".to_string()));
    println!("{:?}", tir);
    Ok(())
}
