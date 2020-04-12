use std::io::Result;
use std::path::Path;

mod lexer;
use lexer::tokens;

mod parser;
use ir::hir::ChannelIr;
use parser::Parser;

use std::sync::mpsc::channel;

pub async fn begin_parsing(path: &Path) -> Result<()> {
    println!("Beginning parsing");
    let instr = std::fs::read_to_string(&path)?;
    let (mut lexer, rx) = lexer::Lexer::new(instr.as_str())?;
    println!("About to start tokenizing");
    let (ir_tx, ir_rx) = channel::<Option<ChannelIr>>();
    let mut parser = Parser::new(
        path.file_stem().unwrap().to_str().unwrap().to_string(),
        ir_tx,
        rx,
    );
    
    let (lexer_result, parser_result) = futures::join!(lexer.start_tokenizing(), parser.parse());

    lexer_result.expect("Failed to tokenize");
    parser_result.expect("An error occurred while parsing module.");

    for ir in ir_rx {
        println!("{:?}", ir.unwrap());
    }
    Ok(())
}
