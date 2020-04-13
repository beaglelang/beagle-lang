use std::io::Result;
use std::path::Path;

pub mod lexer;
use lexer::tokens;

mod parser;
use ir::hir::ChannelIr;
use parser::Parser;
use core::pos::BiPos;

use std::sync::mpsc::channel;

use notices::{
    Notice,
    NoticeLevel
};

pub struct Driver;

impl Driver{
    
    pub async fn begin_parsing(&self, path: &Path) -> ir::hir::Module {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let instr = std::fs::read_to_string(&path).unwrap();
        
        let (token_tx, token_rx) = channel::<tokens::LexerToken>();
        let (ir_tx, ir_rx) = channel::<Option<ChannelIr>>();
        let (notice_tx, notice_rx) = channel::<Option<Notice>>();
    
        let mut lexer = lexer::Lexer::new(instr.as_str(), token_tx.clone()).unwrap();
        let parser_task = Parser::parse(name.clone(), ir_tx, token_rx, notice_tx);
        let mut tir = ir::hir::Module::new(name.clone());
    
        let lexer_task = lexer.start_tokenizing();
    
        let notice_task = async{
            loop{
                match notice_rx.recv(){
                    Ok(Some(n)) => {
                        match n.level{
                            NoticeLevel::Halt => break,
                            _ => n.report(Some(instr.clone().as_str()))
                        };
                    },
                    Ok(None) => continue,
                    Err(m) => {
                        println!("An error occurred while receiving notice from parser: {:?}", m.to_string());
                        break;
                    }
                };
            }
        };
        
        
        let ir_task = async{
            while let Ok(Some(ir)) = ir_rx.recv() {
                match ir.ins{
                    ir::hir::Instruction::Halt => break,
                    _ => tir.push(ir.pos, ir.sig, ir.ins)
                };
            }
        };
        let (lexer_result, parser_result, _, _) = futures::join!(lexer_task, parser_task, notice_task, ir_task);
    
        lexer_result.unwrap();
        parser_result.unwrap();
    
        tir
    }
}

