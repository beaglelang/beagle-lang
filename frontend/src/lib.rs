use std::io::Result;
use std::path::Path;

mod lexer;
use lexer::tokens;

mod parser;
use parser::Parser;

pub async fn begin_parsing(path: &Path) -> Result<()> {
    println!("Beginning parsing");
    let instr = std::fs::read_to_string(&path)?;
    let (mut lexer, rx) = lexer::Lexer::new(instr.as_str())?;
    println!("About to start tokenizing");
    lexer
        .start_tokenizing()
        .await
        .expect("Failed to complete tokenization");
    for token in rx.iter() {
        match token.type_{
            tokens::TokenType::Eof => break,
            _ => println!("{}", token)
        }
    }
    println!("Successfully parsed {:?}", path.file_name());
    Ok(())
}
