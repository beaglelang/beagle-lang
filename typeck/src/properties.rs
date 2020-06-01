use super::{
    Typeck,
    Load,
    Unload,
    ty::{
        GetTy,
        Inference
    }
};

use ty::{
    Ty,
};

use mutable::Mutability;

use expr::{
    Expr,
};

use ident::Identifier;

use std::cell::RefCell;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{
    WriteInstruction
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

use stmt::{
    property::Property
};

impl Inference for Property{
    fn infer_type(&self, typeck: &Typeck) -> Result<(),()> {
        let ty_inner = self.ty.clone().into_inner();
        let expr_ty = self.expr.get_ty();
        if ty_inner.ident == "Unknown"{
            self.ty.replace(Ty{
                ident: expr_ty.ident.clone(),
                pos: ty_inner.pos,
            });
            return Ok(());
        }
        if ty_inner != *expr_ty{
            let ty_source = match typeck.request_source_snippet(ty_inner.pos){
                Ok(source) => source,
                Err(diag) => {
                    typeck.emit_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let expr_source = match typeck.request_source_snippet(self.expr.pos){
                Ok(source) => source,
                Err(diag) => {
                    typeck.emit_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let ty_diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), ty_inner.pos.start.0)
                .level(DiagnosticLevel::Error)
                .message(format!(
                    "Expected an assignment of type {:?}", 
                    ty_inner.ident,
                ))
                .source(ty_source)
                .range(ty_inner.pos.col_range())
                .build();
            let expr_diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), ty_inner.pos.start.0)
            .level(DiagnosticLevel::Error)
            .message(format!(
                "But instead found an assignment of type {:?}", 
                expr_ty.ident,
            ))
            .source(expr_source)
            .range(ty_inner.pos.col_range())
            .build();
            typeck.emit_diagnostic(&[], &[ty_diag_source, expr_diag_source]);
            return Err(())
        }
        Ok(())
    }
}

impl Load for Property{
    type Output = Property;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let mutable = match Mutability::load(chunk, typeck){
            Ok(Some(mutable)) => mutable,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(Some(ident)) => ident,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let ty = match Ty::load(chunk, typeck){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = typeck.chunk_rx.recv(){
            expr_chunk
        }else{
            let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("Failed to get HIR chunk for expression while loading property"))
                .build();
            typeck.emit_diagnostic(&[], &[diag_source]);
            return Err(())
        };
        let expr = match Expr::load(&expr_chunk, typeck){
            Ok(Some(expr)) => expr,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        return Ok(Some(
            Property{
                ident,
                pos,
                ty: RefCell::new(ty),
                expr,
                mutable
            }
        ))
    }
}

impl<'a> super::Check<'a> for Property{
    fn check(&self, typeck: &'a Typeck) -> Result<(),()>{
        self.infer_type(typeck)?;
        Ok(())
    }
}

impl Unload for Property{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Property);
        chunk.write_pos(self.pos);
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.mutable.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.ty.clone().into_inner().unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.expr.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        Ok(chunk)
    }
}