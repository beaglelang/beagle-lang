use std::io::Result;
use std::path::Path;

use ir::{
    Chunk,
};
use lexer::tokens;

use std::sync::mpsc::{
    channel,
};

use notices::{DiagnosticLevel};
use typeck::{
    TypeckManager
};

use module_messages::ModuleMessage;

use std::sync::{ Arc, Mutex };

use std::thread;

#[allow(dead_code)]
pub struct Driver{
    lexer_manager: lexer::LexerManager,
    parser_manager: parser::ParseManager,
    typeck_manager: typeck::TypeckManager,
    memmy_manager: memmy::MemmyManager,
}

impl Driver {
    pub fn new() -> Driver{
        let (_token_tx, _token_rx) = channel::<tokens::LexerToken>();
        let (_hir_tx, _hir_rx) = channel::<Option<Chunk>>();
        let (_typeck_tx, _typeck_rx) = channel::<Option<Chunk>>();
        let (_mir_tx, _mir_rx) = channel::<Option<Chunk>>();
        

        let lexer_manager = lexer::LexerManager::new();
        let parser_manager = parser::ParseManager::new();
        let typeck_manager = TypeckManager::new();
        let memmy_manager = memmy::MemmyManager::new();
        
        Driver{
            lexer_manager,
            parser_manager,
            typeck_manager,
            memmy_manager,
        }
    }

    pub async fn parse_module(&self, path_str: String) -> Result<Box<ir::Module>> {
        let path = Path::new(&path_str);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let path_owned = path.to_owned();
        let read_in = std::fs::read_to_string(path_owned);
        let instr = read_in.unwrap();

        let (diagnostics_tx, diagnostics_rx) = channel();
        let (master_in_tx, master_in_rx) = channel::<ModuleMessage>();
        let (master_out_tx, master_out_rx) = channel::<ModuleMessage>();
        let master_out_rx_arc = Arc::new(Mutex::new(master_out_rx));

        let (token_tx, token_rx) = channel();
        let (hir_tx, hir_rx) = channel();
        let (typeck_tx, typeck_rx) = channel::<Option<Chunk>>();
        let (mir_tx, mir_rx) = channel::<Option<Chunk>>();

        #[allow(unused_mut)]
        let mut module = ir::Module::new(name.clone().to_string());

        self.lexer_manager.enqueue_module(name.clone().to_string(), instr.clone(), diagnostics_tx.clone(), token_tx, master_in_tx.clone(), master_out_rx_arc.clone());
        self.parser_manager.enqueue_module(name.clone().to_string(), diagnostics_tx.clone(), token_rx, hir_tx, master_in_tx.clone(), master_out_rx_arc.clone());
        self.typeck_manager.enqueue_module(name.clone().to_string(), diagnostics_tx.clone(), hir_rx, typeck_tx, master_in_tx.clone(), master_out_rx_arc.clone());
        self.memmy_manager.enqueue_module(name.clone().to_string(), diagnostics_tx.clone(), typeck_rx, mir_tx, master_in_tx.clone(), master_out_rx_arc.clone());
        
        let receive_diagnostics = thread::spawn(move ||{
            while let Ok(Some(n)) = diagnostics_rx.recv(){
                match n.level {
                    DiagnosticLevel::Halt => return,
                    _ => {
                        n.display()
                    },
                }
            }
        });

        let master_communication = thread::spawn(move ||{
            while let Ok(ModuleMessage::SourceRequest(pos)) = master_in_rx.recv(){
                let source = instr.clone().lines().skip(pos.line_region.0).take(pos.line_region.1 - pos.line_region.0).collect();
                master_out_tx.send(ModuleMessage::SourceResponse(source)).unwrap();
            }
        });

        let _ = receive_diagnostics.join();
        let _ = master_communication.join();
        
        Ok(Box::new(module))
    }
}
