extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Instruction)]
pub fn ins_derive(input: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote!{
        impl Instruction for #name{}
    };
    gen.into()
}

#[proc_macro_derive(WriteInstruction)]
pub fn write_ins_derive(input: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote!{
        impl WriteInstruction<#name> for Chunk{
            fn write_instruction(&mut self, ins: #name) {
                self.code.push(ins as u8);
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(ReadInstruction)]
pub fn read_ins_derive(input: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote!{
        impl ReadInstruction<#name> for Chunk{
            fn read_instruction(&self) -> Option<#name> {
                let ins = FromPrimitive::from_u8(self.get_current());
                // self.advance();
                ins
            }
        }
    };
    gen.into()
}
