// #![feature(async_closure)]

use frontend::Driver;

fn main() -> std::io::Result<()> {
    let driver = Driver::new();
    let _ = futures::executor::block_on(driver.parse_module("test.txt".to_string()));
    Ok(())
}
