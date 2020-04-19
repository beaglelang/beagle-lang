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

use std::sync::mpsc::{
    channel,
    Sender,
    Receiver
};

use notices::{Notice, NoticeLevel};
use typeck::{
    Typeck,
    TypeckManager
};

trait ModuleWrapper<'a>{}
impl<'a> ModuleWrapper<'a> for ir::Module{}

pub struct Driver{
    lexer_manager: lexer::LexerManager,
    parser_manager: parser::ParseManager,
    typeck_manager: typeck::TypeckManager,
    notice_rx: Receiver<Option<Notice>>
}

impl<'a> Driver {
    pub fn new() -> Driver{
        let (token_tx, token_rx) = channel::<tokens::LexerToken>();
        let (hir_tx, hir_rx) = channel::<Option<HIR>>();
        let (notice_tx, notice_rx) = channel::<Option<Notice>>();
        let (typeck_tx, typeck_rx) = channel::<Option<HIR>>();
        let (mir_tx, mir_rx) = channel::<Option<MIR>>();

        let lexer_manager = lexer::LexerManager::new(notice_tx.clone());
        let parser_manager = parser::ParseManager::new(notice_tx.clone());
        let typeck_manager = TypeckManager::new(notice_tx.clone());
        // let mut tir = ir::Module::new(name.clone());
        // let typeck_task = TypeckVM::start_checking(name.clone(), ir_rx, notice_tx.clone(), typeck_tx);
        // let memmy_task = memmy::MemmyGenerator::start(name.clone(), mir_tx, notice_tx, typeck_rx);
        Driver{
            lexer_manager,
            parser_manager,
            typeck_manager,
            notice_rx
        }
    }

    pub async fn parse_module(&self, path: &'static Path) -> Result<Box<ir::Module>> {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let instr = std::fs::read_to_string(&path).unwrap().as_str();

        let (token_tx, token_rx) = channel();
        let (hir_tx, hir_rx) = channel();
        let (typeck_tx, typeck_rx) = channel::<HIR>();
        let (mir_tx, mir_rx) = channel::<MIR>();

        let mut module = ir::Module::new(name.clone().to_string());

        self.lexer_manager.enqueue_module(name.clone().to_string(), instr.clone(), token_tx);
        self.parser_manager.enqueue_module(name.clone().to_string(), token_rx, hir_tx);
        

        let notice_task = async {
            loop {
                match self.notice_rx.recv() {
                    Ok(Some(n)) => {
                        match n.level {
                            NoticeLevel::Halt => break,
                            _ => n.report(Some(instr.clone())),
                        };
                    },
                    Ok(_) => continue,
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
            while let Ok(ir) = mir_rx.recv() {
                match ir.ins {
                    MIRInstruction::Halt => {
                        &module.push(ir.pos, ir.sig, ir.ins);
                        break
                    },
                    _ => {
                        &module.push(ir.pos, ir.sig, ir.ins);
                    },
                };
            }
        };

        futures::join!(ir_task);
        
        Ok(Box::new(module))
    }
}
