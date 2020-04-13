use serde::{Deserialize, Serialize};

use core::pos::BiPos;

pub mod hir;
pub mod type_signature;

const TAB_WIDTH: usize = 5;

pub trait SourceOrigin{
    fn locate_in_source(self, source: &str) -> Option<(usize, Vec<&str>, String)>;
}

impl SourceOrigin for BiPos {
    fn locate_in_source(self, source: &str) -> Option<(usize, Vec<&str>, String)> {
        let start_line = if self.start.0 > 3 {
            self.start.0 - 3
        } else {
            1
        };
        let lines: Vec<&str> = source
            .lines()
            .skip(start_line as usize - 1)
            .take(7)
            .collect();
        let error_line = if lines.len() > 3 {
            lines[3]
        } else {
            lines[self.start.0 as usize - 1]
        };
        let tab_count = error_line
            .chars()
            .enumerate()
            .filter(|(i, c)| *c == '\t' && *i < self.start.1 as usize - 1)
            .count();
        let squiggly_line = format!(
            "{}{}{}",
            String::from("~").repeat(self.start.1 as usize - 1 + tab_count * TAB_WIDTH),
            "^",
            if (self.start.1 as usize) < error_line.len() {
                String::from("~").repeat(error_line.len() - self.end.1 as usize)
            } else {
                String::new()
            }
        );
        Some((start_line as usize, lines, squiggly_line))
    }
}
