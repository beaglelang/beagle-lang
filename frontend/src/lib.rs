use std::io::Result;
use std::path::Path;

pub mod lexer;
use lexer::tokens;

pub mod parser;
use ir::{
    Chunk,
};
use parser::Parser;

use std::sync::mpsc::{
    channel,
    Receiver
};

use notices::{Notice, NoticeLevel};
use typeck::{
    TypeckManager
};

trait ModuleWrapper<'a>{}
impl<'a> ModuleWrapper<'a> for ir::Module{}

#[allow(dead_code)]
pub struct Driver{
    lexer_manager: lexer::LexerManager,
    parser_manager: parser::ParseManager,
    typeck_manager: typeck::TypeckManager,
    notice_rx: Receiver<Option<Notice>>
}

impl Driver {
    pub fn new() -> Driver{
        let (_token_tx, _token_rx) = channel::<tokens::LexerToken>();
        let (_hir_tx, _hir_rx) = channel::<Option<Chunk>>();
        let (notice_tx, notice_rx) = channel::<Option<Notice>>();
        let (_typeck_tx, _typeck_rx) = channel::<Option<Chunk>>();
        let (_mir_tx, _mir_rx) = channel::<Option<Chunk>>();

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

    pub async fn parse_module(&self, path_str: String) -> Result<Box<ir::Module>> {
        let path = Path::new(&path_str);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let path_owned = path.to_owned();
        let read_in = std::fs::read_to_string(path_owned);
        let instr = read_in.as_ref().unwrap();

        let (token_tx, token_rx) = channel();
        let (hir_tx, hir_rx) = channel();
        let (_typeck_tx, _typeck_rx) = channel::<Option<Chunk>>();
        let (_mir_tx, _mir_rx) = channel::<Option<Chunk>>();

        #[allow(unused_mut)]
        let mut module = ir::Module::new(name.clone().to_string());

        self.lexer_manager.enqueue_module(name.clone().to_owned(), instr.clone(), token_tx);
        self.parser_manager.enqueue_module(name.clone().to_string(), token_rx, hir_tx);
        

        let notice_task = async {
            loop {
                match self.notice_rx.recv() {
                    Ok(Some(n)) => {
                        match n.level {
                            NoticeLevel::Halt => break,
                            _ => n.report(Some(instr.clone().as_str())),
                        };
                    },
                    Ok(_) => continue,
                    Err(_) => {
                        break;
                    }
                };
            }
        };

        // let ir_task = async {
        //     while let Ok(Some(chunk)) = mir_rx.recv() {
        //         println!("{}", chunk);
        //     }
        // };

        let parser_ir_task = async{
            while let Ok(Some(chunk)) = hir_rx.recv() {
                println!("{}", chunk);
            }
        };

        futures::join!(notice_task, parser_ir_task);
        
        Ok(Box::new(module))
    }
}
