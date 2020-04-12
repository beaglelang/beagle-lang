use std::io::Result;
use std::path::Path;

mod lexer;
use lexer::tokens;

mod parser;
use parser::Parser;
use ir::hir::ChannelIr;

use std::sync::mpsc::channel;

pub async fn begin_parsing(path: &Path) -> Result<()> {
    println!("Beginning parsing");
    let instr = std::fs::read_to_string(&path)?;
    let (mut lexer, rx) = lexer::Lexer::new(instr.as_str())?;
    println!("About to start tokenizing");
    let lexer_task = async{
        lexer
            .start_tokenizing()
            .await
            .expect("Failed to complete tokenization");
    };
    let (ir_tx, ir_rx) = channel::<Option<ChannelIr>>();
    let mut parser = Parser::new(path.file_stem().unwrap().to_str().unwrap().to_string(), ir_tx, rx);
    let parse_task = async{
        parser.parse().await.expect("An error occurred while parsing module.");
    };
    futures::join!(lexer_task, parse_task);
    for ir in ir_rx{
        println!("{:?}", ir.unwrap());
    }
    Ok(())
}
