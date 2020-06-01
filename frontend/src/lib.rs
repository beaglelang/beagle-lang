use std::io::Result;
use std::path::Path;

use ir::{
    Chunk,
};
use lexer::tokens;

use std::sync::mpsc::{
    channel,
    Receiver
};

use notices::{Diagnostic, DiagnositcBuilder, DiagnosticLevel};
use typeck::{
    TypeckManager
};

enum ModuleMessage{
    SourceRequest,
    SourceResponse(BiPos),
}

struct ModuleComms{
    module_name: String,
    send: SendChannel<ModuleMessage>,
    recv: ReceiveChannel<ModuleMessage>
}

#[allow(dead_code)]
pub struct Driver{
    lexer_manager: lexer::LexerManager,
    parser_manager: parser::ParseManager,
    typeck_manager: typeck::TypeckManager,
    memmy_manager: memmy::MemmyManager,
    notice_rx: Receiver<Option<Diagnostic>>,
    master_tx: Sender<ModuleMessage>,
    master_rx: Sender<ModuleMessage>,
    module_comm_channels: Vec<ModuleComms>, 
}

impl Driver {
    pub fn new() -> Driver{
        let (_token_tx, _token_rx) = channel::<tokens::LexerToken>();
        let (_hir_tx, _hir_rx) = channel::<Option<Chunk>>();
        let (notice_tx, notice_rx) = channel::<Option<Notice>>();
        let (_typeck_tx, _typeck_rx) = channel::<Option<Chunk>>();
        let (_mir_tx, _mir_rx) = channel::<Option<Chunk>>();
        let (master_tx, master_rx) = channel::<ModuleMessage>();

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
            master_tx, master_rx,
            module_comm_channels: HashMap::<String, Channel>::new()
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

        self.lexer_manager.enqueue_module(name.clone().to_string(), instr.clone(), token_tx);
        self.parser_manager.enqueue_module(name.clone().to_string(), token_rx, hir_tx);
        self.typeck_manager.enqueue_module(name.clone().to_string(), hir_rx, typeck_tx);
        self.memmy_manager.enqueue_module(name.clone().to_string(), typeck_rx, mir_tx);

        let notice_task = async {
            loop {
                match self.notice_rx.recv() {
                    Ok(Some(n)) => {
                        match n.level {
                            NoticeLevel::Halt => break,
                            _ => {
                                
                            },
                        };
                    },
                    Ok(_) | Err(_) => break,
                };
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

        futures::join!(ir_task, notice_task);
        
        Ok(Box::new(module))
    }
}
