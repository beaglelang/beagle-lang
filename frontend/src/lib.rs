use std::io::Result;
use std::path::Path;

use ir::{
    Chunk,
};
use lexer::tokens;

use std::sync::mpsc::{
    channel,
    Sender,
    Receiver
};

use notices::{Diagnostic, DiagnosticLevel};
use typeck::{
    TypeckManager
};

use module_messages::ModuleMessage;

use std::sync::{ Arc, Mutex };

#[allow(dead_code)]
pub struct Driver{
    lexer_manager: lexer::LexerManager,
    parser_manager: parser::ParseManager,
    typeck_manager: typeck::TypeckManager,
    memmy_manager: memmy::MemmyManager,
    notice_rx: Receiver<Option<Diagnostic>>,
    master_in_tx: Sender<ModuleMessage>,
    master_in_rx: Receiver<ModuleMessage>,
    master_out_tx: Sender<ModuleMessage>,
    master_out_rx: Arc<Mutex<Receiver<ModuleMessage>>>
}

impl Driver {
    pub fn new() -> Driver{
        let (_token_tx, _token_rx) = channel::<tokens::LexerToken>();
        let (_hir_tx, _hir_rx) = channel::<Option<Chunk>>();
        let (notice_tx, notice_rx) = channel::<Option<Diagnostic>>();
        let (_typeck_tx, _typeck_rx) = channel::<Option<Chunk>>();
        let (_mir_tx, _mir_rx) = channel::<Option<Chunk>>();
        let (master_in_tx, master_in_rx) = channel::<ModuleMessage>();
        let (master_out_tx, master_out_rx) = channel::<ModuleMessage>();

        let lexer_manager = lexer::LexerManager::new(notice_tx.clone());
        let parser_manager = parser::ParseManager::new(notice_tx.clone());
        let typeck_manager = TypeckManager::new(notice_tx.clone());
        let memmy_manager = memmy::MemmyManager::new(notice_tx.clone());
        
        Driver{
            lexer_manager,
            parser_manager,
            typeck_manager,
            memmy_manager,
            notice_rx,
            master_in_tx,
            master_in_rx,
            master_out_tx,
            master_out_rx: Arc::new(Mutex::new(master_out_rx))
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
        let (typeck_tx, typeck_rx) = channel::<Option<Chunk>>();
        let (mir_tx, mir_rx) = channel::<Option<Chunk>>();

        #[allow(unused_mut)]
        let mut module = ir::Module::new(name.clone().to_string());

        self.lexer_manager.enqueue_module(name.clone().to_string(), instr.clone(), token_tx, self.master_in_tx.clone(), self.master_out_rx.clone());
        self.parser_manager.enqueue_module(name.clone().to_string(), token_rx, hir_tx, self.master_in_tx.clone(), self.master_out_rx.clone());
        self.typeck_manager.enqueue_module(name.clone().to_string(), hir_rx, typeck_tx, self.master_in_tx.clone(), self.master_out_rx.clone());
        self.memmy_manager.enqueue_module(name.clone().to_string(), typeck_rx, mir_tx, self.master_in_tx.clone(), self.master_out_rx.clone());

        let notice_task = async {
            loop {
                match self.notice_rx.recv() {
                    Ok(Some(n)) => {
                        match n.level {
                            DiagnosticLevel::Halt => break,
                            _ => {
                                n.display()
                            },
                        };
                    },
                    Ok(_) | Err(_) => break,
                };
            }
        };

        let master_channel_listener = async{
            while let Ok(message) = self.master_in_rx.recv(){
                match message{
                    ModuleMessage::SourceRequest(pos) => {
                        let source = instr.clone().split_off(pos.offset.0).split_off(pos.offset.1);
                        self.master_out_tx.send(ModuleMessage::SourceResponse(source)).unwrap();
                    }
                    _ => {
                        println!("Line 110 in frontend module was triggered somehow, this should not have happened.");
                        break;
                    }
                }
            }
        };

        let ir_task = async {
            while let Ok(Some(chunk)) = mir_rx.recv() {
                println!("{}", chunk);
            }
        };

        // let parser_ir_task = async{
        //     while let Ok(Some(chunk)) = typeck_rx.recv() {
        //         println!("{:?}", chunk);
        //     }
        // };

        futures::join!(notice_task, master_channel_listener, ir_task);
        
        Ok(Box::new(module))
    }
}
