use std::io::Result;
use std::path::Path;

pub mod lexer;
use lexer::tokens;

pub mod parser;
use core::pos::BiPos;
use ir::{
    hir::HIR,
    mir::{
        MIR,
        MIRInstruction
    },
};
use parser::Parser;

use std::sync::mpsc::channel;

use notices::{Notice, NoticeLevel};
use typeck::TypeckVM;

pub struct Driver;

impl Driver {
    pub async fn begin_parsing(&self, path: &Path) -> ir::Module {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let instr = std::fs::read_to_string(&path).unwrap();

        let (token_tx, token_rx) = channel::<tokens::LexerToken>();
        let (ir_tx, ir_rx) = channel::<Option<HIR>>();
        let (notice_tx, notice_rx) = channel::<Option<Notice>>();
        let (typeck_tx, typeck_rx) = channel::<Option<HIR>>();
        let (mir_tx, mir_rx) = channel::<Option<MIR>>();
        
        let mut lexer = lexer::Lexer::new(instr.as_str(), token_tx.clone()).unwrap();
        let parser_task = Parser::parse(name.clone(), ir_tx, token_rx, notice_tx.clone());
        let mut tir = ir::Module::new(name.clone());
        let typeck_task = TypeckVM::start_checking(name.clone(), ir_rx, notice_tx.clone(), typeck_tx);
        let memmy_task = memmy::MemmyGenerator::start(name.clone(), mir_tx, notice_tx, typeck_rx);

        let lexer_task = lexer.start_tokenizing();

        let notice_task = async {
            loop {
                match notice_rx.recv() {
                    Ok(Some(n)) => {
                        match n.level {
                            NoticeLevel::Halt => break,
                            _ => n.report(Some(instr.clone().as_str())),
                        };
                    }
                    Ok(None) => continue,
                    Err(m) => {
                        println!(
                            "An error occurred while receiving notice from parser: {:?}",
                            m.to_string()
                        );
                        break;
                    }
                };
            }
        };

        let ir_task = async {
            while let Ok(Some(ir)) = mir_rx.recv() {
                match ir.ins {
                    MIRInstruction::Halt => {
                        tir.push(ir.pos, ir.sig, ir.ins);
                        break
                    },
                    _ => {
                        tir.push(ir.pos, ir.sig, ir.ins);
                    },
                };
            }
        };
        let (lexer_result, parser_result, typeck_result, memmy_result, _, _) =
            futures::join!(lexer_task, parser_task,typeck_task, memmy_task, notice_task, ir_task);
        // let (lexer_result, parser_result, _, _) =
        //     futures::join!(lexer_task, parser_task, notice_task, ir_task);

        lexer_result.unwrap();
        parser_result.unwrap();
        typeck_result.unwrap();
        memmy_result.unwrap();

        tir
    }
}
