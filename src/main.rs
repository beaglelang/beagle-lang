// #![feature(async_closure)]

use frontend::Driver;

use std::path::Path;
use std::thread;

fn main() -> std::io::Result<()> {
    let driver = Driver;
    let tir = futures::executor::block_on(driver.begin_parsing(Path::new("test.txt")));
    println!("{:?}", tir);
    Ok(())
}
