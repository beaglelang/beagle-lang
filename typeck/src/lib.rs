use ir::{
    hir::{
        HIRInstruction
    },
    Chunk,
};

use notices::*;
use std::sync::{
    mpsc::{
        Sender, Receiver
    },
};

use core::pos::BiPos;

use ir_traits::{
    ReadInstruction,
    WriteInstruction,
};

use futures::executor::ThreadPool;

use core::pos::BiPos as Position;

pub struct TypeckManager{
    thread_pool: ThreadPool,
    notice_tx: Sender<Option<Notice>>,
}

impl TypeckManager{
    pub fn new(notice_tx: Sender<Option<Notice>>) -> Self{
        TypeckManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx,
        }
    }

    pub fn enqueue_module(&self, module_name: String, hir_rx: Receiver<Option<Chunk>>, typeck_tx: Sender<Option<Chunk>>){
        let notice_tx_clone = self.notice_tx.clone();
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let typeck = Typeck::start_checking(module_name_clone.clone(), hir_rx, notice_tx_clone.clone(), typeck_tx);
            if let Err(msg) = typeck{
                let notice = Notice{
                    from: "Typeck".to_string(),
                    file: module_name_clone,
                    level: NoticeLevel::Error,
                    msg,
                    pos: Position::default()
                };
                notice_tx_clone.clone().send(Some(notice)).unwrap();
            };
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TyValueElement{
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Custom(String),
    Unit
}

#[derive(Debug, Clone, PartialEq)]
struct TyValue{
    ty: Ty,
    elem: TyValueElement
}

trait Check<'a>{
    fn check(&self, analyzer: &'a TypeckAnalyzer) -> Result<(), ()>;
}

#[derive(Debug, Clone, PartialEq)]
struct Ty{
    ident: String,
    pos: BiPos
}

#[derive(Debug, Clone)]
struct Fun{
    ident: String,
    ty: Ty,
    params: Vec<FunParam>,
    body: Vec<Statement>,
    pos: BiPos,
}

#[derive(Debug, Clone)]
struct FunParam{
    ident: String,
    ty: Ty,
    pos: BiPos
}

impl<'a> Check<'a> for Fun{
    fn check(&self, analyzer: &'a TypeckAnalyzer) -> Result<(), ()> {
        for stat in self.body.iter(){
            match &stat.kind{
                StatementKind::TerminalRet(expr) => {
                    match expr.kind.as_ref(){
                        ExprElement::Value(ty) => {
                            if ty.ty != self.ty{
                                analyzer.emit_type_error(self.ty.clone(), ty.ty.clone());
                                return Err(())
                            }
                        }
                        _ => continue
                    }
                }
                _ => continue
            }
        }
        Ok(())
    }
}

trait GetTy{
    fn get_ty(&self) -> &Ty;
}

#[derive(Debug, Clone)]
struct Statement{
    kind: StatementKind,
    pos: BiPos,
}

struct Module{
    ident: String,
    statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
struct Property{
    ident: String,
    ty: Ty,
    expr: Expr,
    pos: BiPos,
}

#[derive(Debug, Clone)]
struct Local{
    ident: String,
    ty: Ty,
    expr: Expr,
    pos: BiPos,
}

impl<'a> Check<'a> for Local{
    fn check(&self, analyzer: &TypeckAnalyzer) -> Result<(), ()> {
        if &self.ty == self.expr.get_ty(){
            return Ok(())
        }
        analyzer.emit_type_error(self.ty.clone(), self.expr.get_ty().clone());
        Err(())
    }
}

#[derive(Debug, Clone)]
enum StatementKind{
    Property(Property),
    Fun(Fun),
    Local(Local),
    TerminalRet(Expr)
}

#[derive(Debug, Clone)]
struct Expr{
    kind: Box<ExprElement>,
    pos: BiPos,
}

impl GetTy for Expr{
    fn get_ty(&self) -> &Ty {
        self.kind.get_ty()
    }
}

#[derive(Debug, Clone)]
enum ExprElement{
    Grouped(Expr),
    Value(TyValue),
    UnaryOp(TyValue, TyValue)
}

impl GetTy for ExprElement{
    fn get_ty(&self) -> &Ty{
        match self{
            Self::Grouped(expr) => {
                expr.get_ty()
            }
            Self::Value(t) => &t.ty,
            Self::UnaryOp(left, _) => &left.ty
        }
    }
}

#[derive(Debug)]
struct TypeckElement{
    statement: Statement,
    pos: BiPos,
}

struct TypeckAnalyzer{
    module_name: String,
    elements: Vec<TypeckElement>,
    notice_tx: Sender<Option<Notice>>,
}

impl TypeckAnalyzer{
    fn new(module_name: String, notice_tx: Sender<Option<Notice>>) -> Self{
        Self{
            module_name,
            elements: vec![],
            notice_tx
        }
    }

    fn emit_notice(&self, msg: String, level: NoticeLevel, pos: Position) -> Result<(),()>{
        if self.notice_tx.send(
            Some(notices::Notice{
                from: "Type checker".to_string(),
                msg,
                file: self.module_name.clone().to_string(),
                level,
                pos
            })
        ).is_err(){
            return Err(())
        }
        Ok(())
    }

    fn emit_type_error(&self, ty1: Ty, ty2: Ty){
        self.emit_notice(format!("Expected a value of type {:?}", ty1.clone()), NoticeLevel::Error, ty1.pos).unwrap();
        self.emit_notice(format!("But instead got value of type {:?}", ty2.clone()), NoticeLevel::Error, ty2.pos).unwrap();
    }
}

pub struct Typeck{
    module_name: String,
    chunk_rx: Receiver<Option<Chunk>>,
    notice_tx: Sender<Option<Notice>>,
    typeck_tx: Sender<Option<Chunk>>,
    ty_analyzer: TypeckAnalyzer,
}

impl<'a> Typeck{
    fn emit_notice(&self, msg: String, level: NoticeLevel, pos: Position) -> Result<(),()>{
        if self.notice_tx.send(
            Some(notices::Notice{
                from: "Type checker".to_string(),
                msg,
                file: self.module_name.clone(),
                level,
                pos
            })
        ).is_err(){
            return Err(())
        }
        Ok(())
    }

    ///We need to keep track of the expression chunk and return it after we type check it.
    fn load_expression(&self, chunk: Chunk) -> Result<Expr,()>{
        let ins: Option<HIRInstruction> = chunk.read_instruction();
        let pos = chunk.read_pos();
        match &ins {
            Some(HIRInstruction::Bool) => {
                let value = chunk.read_bool();
                let kind = ExprElement::Value(TyValue{
                    ty: Ty{
                        ident: "Bool".to_owned(),
                        pos
                    },
                    elem: TyValueElement::Bool(value),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::Integer) => {
                let value = chunk.read_int();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Integer(value),
                    ty: Ty{
                        ident: "Integer".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::Float) => {
                let value = chunk.read_float();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Float(value),
                    ty: Ty{
                        ident: "Float".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::String) => {
                let value = chunk.read_string().to_owned();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::String(value),
                    ty: Ty{
                        ident: "String".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            _ => {
                self.emit_notice(format!("Expected an expression but instead got instruction {:?}", ins), NoticeLevel::Error, pos).unwrap();
                return Err(());
            }
        }
    }

    fn load_property(&self, chunk: Chunk) -> Result<Property,()>{
        let pos = chunk.read_pos();
        let mutable = chunk.read_bool();
        let name = chunk.read_string().to_string();
        let current_type: HIRInstruction = chunk.read_instruction().unwrap();
        let typename = if current_type == HIRInstruction::Custom{
            let typename = chunk.read_string().to_owned();
            Some(typename)
        }else{
            None
        };
        let expr_chunk = if let Ok(Some(expr_chunk)) = self.chunk_rx.recv(){
            expr_chunk
        }else{
            return Err(())
        };
        
        let expr = match self.load_expression(expr_chunk){
            Ok(expr) => expr,
            Err(()) => return Err(())
        };
        let property = Property{
            ident: name.clone(),
            ty: Ty{
                ident: if typename.is_some(){
                    typename.unwrap()
                }else{
                    match &current_type{
                        HIRInstruction::Integer => "Int".to_owned(),
                        HIRInstruction::String => "String".to_owned(),
                        HIRInstruction::Float => "Float".to_owned(),
                        HIRInstruction::Bool => "Bool".to_owned(),
                        HIRInstruction::Unknown => "Unknown".to_owned(),
                        _ => {
                            self.emit_notice(format!("Unrecognized type element; this is a bug in the compiler: {:?}", current_type), NoticeLevel::Error, pos).unwrap();
                            return Err(())
                        },
                    }
                },
                pos: pos.clone(),
            },
            expr,
            pos
        };
        Ok(property)
    }

    fn load_function(&self, chunk: Chunk) -> Result<Fun,()>{
        let pos = chunk.read_pos();
        let name = chunk.read_string();
        let mut params = vec![];
        while let Some(ins) = chunk.read_instruction() as Option<HIRInstruction>{
            if ins != HIRInstruction::EndParams{
                break;
            }

            let pos = chunk.read_pos();
            if ins != HIRInstruction::FnParam{
                self.emit_notice(format!("Expected an fn param instruction but instead got {:?}; this is a bug in the compiler.", ins), NoticeLevel::Error, pos)?;
                return Err(())
            }

            let param_name = chunk.read_string();
            let param_type = chunk.read_instruction() as Option<HIRInstruction>;
            let param_typename = match param_type{
                Some(type_) => {
                    if type_ == HIRInstruction::Custom{
                        Some(chunk.read_string())
                    }else{
                        None
                    }
                }
                None => {
                    self.emit_notice(format!("Expected a param type annotation but instead got none. This is a bug in the compiler."), NoticeLevel::Error, pos)?;
                    return Err(())
                }
            };
            params.push(FunParam{
                ident: param_name.to_owned(),
                ty: Ty{
                    ident: if param_typename.is_none(){
                        param_typename.unwrap().to_owned()
                    }else{
                        format!("{:?}", param_typename.unwrap())
                    },
                    pos
                },
                pos
            });
        }
        let fun_type_pos = chunk.read_pos();
        let return_type = chunk.read_instruction() as Option<HIRInstruction>;
        let typename = match return_type{
            Some(name_ins) => {
                if name_ins == HIRInstruction::Custom{
                    chunk.read_string().to_owned()
                }else{
                    format!("{:?}", name_ins)
                }
            }
            None => {
                self.emit_notice(format!("Expected a return type instruction but instead got {:?}; this is compiler bug.", return_type.unwrap()), NoticeLevel::Error, fun_type_pos)?;
                return Err(())
            }
        };
        let mut statements = vec![];
        loop{
            let next_chunk = self.chunk_rx.recv().unwrap().unwrap();
            if let Some(HIRInstruction::EndFn) = next_chunk.read_instruction(){
                break;
            }
            let statement = match self.load_statement(next_chunk){
                Ok(statement) => statement,
                Err(()) => return Err(())
            };
            statements.push(statement);
        }
        let fun = Fun{
            ident: name.to_owned(),
            ty: Ty{
                ident: typename,
                pos: fun_type_pos,
            },
            body: Vec::new(),
            params,
            pos
        };
        Ok(fun)
    }

    fn load_statement(&self, chunk: Chunk) -> Result<Statement,()>{
        match chunk.read_instruction(){
            Some(HIRInstruction::Property) => match self.load_property(chunk){
                Ok(property) => {
                    Ok(Statement{
                        kind: StatementKind::Property(property.clone()),
                        pos: property.pos.clone()
                    })
                },
                Err(()) => Err(())
            },
            Some(HIRInstruction::Fn) => match self.load_function(chunk){
                Ok(fun) => {
                    Ok(Statement{
                        kind: StatementKind::Fun(fun.clone()),
                        pos: fun.pos.clone()
                    })
                },
                Err(()) => Err(())
            },
            _ => Err(())
        }
    }

    fn load(&mut self) -> Result<(),()>{
        loop{
            let chunk = if let Ok(Some(chunk)) = self.chunk_rx.recv(){
                chunk
            }else{
                return Ok(())
            };
            let statement = match self.load_statement(chunk){
                Ok(statement) => statement,
                Err(()) => return Err(())
            };
            self.ty_analyzer.elements.push(TypeckElement{
                statement: statement.clone(),
                pos: statement.pos.clone()
            });
        }
    }

    pub fn start_checking(module_name: String, ir_rx: Receiver<Option<Chunk>>, notice_tx: Sender<Option<Notice>>, typeck_tx: Sender<Option<Chunk>>) -> Result<(), String>{
        let mut typeck = Self{
            module_name: module_name.clone(),
            notice_tx: notice_tx.clone(),
            typeck_tx,
            chunk_rx: ir_rx,
            ty_analyzer: TypeckAnalyzer::new(module_name.clone(), notice_tx.clone()),
        };

        if typeck.load().is_err(){
            return Err("An error occurred while loading bytecode into type analyzer".to_owned())
        }
        
        for el in typeck.ty_analyzer.elements.iter(){
            println!("{:?}", el);
        }

        typeck.typeck_tx.send(None).unwrap();
        
        Ok(())
    }
}