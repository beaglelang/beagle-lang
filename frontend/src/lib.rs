use std::io::Result;
use std::path::Path;

mod lexer;
use lexer::tokens;

mod parser;
use ir::hir::ChannelIr;
use parser::Parser;

use std::sync::mpsc::channel;

pub async fn begin_parsing(path: &Path) -> Result<()> {
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let instr = std::fs::read_to_string(&path)?;
    let (token_tx, token_rx) = channel::<tokens::LexerToken>();
    let mut lexer = lexer::Lexer::new(instr.as_str(), token_tx)?;
    let (ir_tx, ir_rx) = channel::<Option<ChannelIr>>();
    let mut parser = Parser::new(
        name.clone(),
        ir_tx,
        token_rx,
    );

    let lexer_task = async{
        lexer.start_tokenizing().await.unwrap();
    };

    let parser_task = async{
        parser.parse().await.unwrap();
    };

    futures::join!(lexer_task, parser_task);

    let mut tir = ir::hir::Module::new(name);

    for ir in ir_rx{
        match ir{
            Some(ir_) => match ir_.ins{
                ir::hir::Instruction::Eof => break,
                _ => tir.push(ir_.pos, ir_.sig, ir_.ins)
            }
            _ => continue
        }
        // tir.push(ir.pos, ir.sig, ir.ins)
    }

    println!("{:?}", tir);

    Ok(())
}
