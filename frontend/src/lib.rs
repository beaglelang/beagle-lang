use std::io::Result;
use std::path::Path;

mod lexer;
// mod parser;

pub async fn begin_parsing(path: &Path) -> Result<()> {
    println!("Beginning parsing");
    let instr = std::fs::read_to_string(&path)?;
    let (mut lexer, rx) = lexer::Lexer::new(instr.as_str())?;
    println!("About to start tokenizing");
    lexer
        .start_tokenizing()
        .expect("Failed to complete tokenization");
    for token in rx.iter() {
        println!("{}", token);
    }
    Ok(())
}
