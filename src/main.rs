// #![feature(async_closure)]

use frontend::begin_parsing;

use std::path::Path;
use std::thread;

fn main() -> std::io::Result<()> {
    thread::spawn(move || {
        let _ = futures::executor::block_on(begin_parsing(Path::new("test.txt")));
    }).join().expect("Unable to parse file");
    Ok(())
}
