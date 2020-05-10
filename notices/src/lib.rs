use core::{
    ansi,
    pos::{BiPos, Position},
};

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum NoticeLevel {
    Notice,
    Warning,
    Error,
    Halt,
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub from: String,
    pub msg: String,
    pub pos: BiPos,
    pub file: String,
    pub level: NoticeLevel,
}

impl Notice {
    pub fn report(self, source: Option<&str>) {
        let (colour, prefix) = match self.level {
            NoticeLevel::Notice => (ansi::Fg::Cyan, "[-]: "),
            NoticeLevel::Warning => (ansi::Fg::Yellow, "[*]: "),
            NoticeLevel::Error => (ansi::Fg::Red, "[!]: "),
            NoticeLevel::Halt => return,
        };

        println!(
            "{}{}{}{}: {}",
            colour,
            prefix,
            self.from,
            ansi::Fg::Reset,
            self.msg
        );

        println!("\tat [{}:{}]\n", self.file, self.pos);

        if self.pos.start != Position::default() && self.pos.end != Position::default() {
            if let Some(src) = source {
                if let Some((start_line, lines, squiggly)) = self.pos.locate_in_source(src) {
                    lines.iter().enumerate().for_each(|(i, line)| {
                        println!(
                            "\t{}{:4}{} | {}",
                            colour,
                            start_line + i,
                            ansi::Fg::Reset,
                            line
                        );

                        if i == start_line {
                            println!("\t{}---- | {}{}", colour, squiggly, ansi::Fg::Reset);
                        };
                    });
                    println!();
                }
            }
        }
    }
}

pub const TAB_WIDTH: usize = 5;

pub trait SourceOrigin {
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
            .skip(start_line as usize)
            .take(7)
            .collect();
        let error_line = if lines.len() > 3 {
            lines[3]
        } else {
            lines[self.start.0 as usize]
        };
        
        let squiggly_line = format!(
            "{}{}",
            core::padding::padding(" ", self.start.1-1),
            if (self.start.1 as usize) < error_line.len() {
                let length = if self.end.1 - self.start.1 == 0{
                    1
                }else{
                    self.end.1 - self.start.1
                };
                core::padding::padding("^", length).to_string()
            } else {
                String::new()
            }
        );
        Some((start_line as usize, lines, squiggly_line))
    }
}
