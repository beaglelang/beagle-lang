use serde::{Deserialize, Serialize};

use core::pos::BiPos;

pub mod hir;
pub mod type_signature;
pub mod mir;

pub const TAB_WIDTH: usize = 5;

use std::fmt::{
    Formatter,
    Result
};

fn repeat_char(c: char, times: usize) -> String{
    std::iter::repeat(c).take(times).collect::<String>()
}

fn fmt_tab(f: &mut Formatter<'_>, depth: usize) -> Result{
    if depth != 0{
        for _ in 0 .. depth{
            write!(f, "|{}", repeat_char(' ', TAB_WIDTH))?;
        }
        Ok(())
    }else{
        Ok(())
    }
}