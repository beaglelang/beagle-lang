use super::{
    SymbolResolver,
    Load,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{ ReadInstruction };

use ty::{
    Ty,
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

impl Load for Ty{
    type Output = Ty;

    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let ins = chunk.read_instruction() as Option<HIRInstruction>;
        let ident = match ins{
            Some(type_) => {
                if type_ == HIRInstruction::Custom{
                    chunk.read_string().to_string()
                }else{
                    format!("{:?}", type_)
                }
            }
            None => {
                let source = match symbol_resolver.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(msg) => {
                        symbol_resolver.emit_diagnostic(&[], &[msg]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Expected a param type annotation but instead got none. This is a bug in the compiler."))
                    .source(source)
                    .build();
                symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        Ok(Some(Ty{
            ident,
            pos
        }))
    }
}