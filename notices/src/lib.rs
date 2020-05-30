use core::{
    ansi,
    pos::{BiPos},
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
///The source location of a notice
pub struct NoticeSource{
    ///File name
    pub file: String,
    ///Location in the file notice is related to
    pub pos: BiPos
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub from: String,
    ///The actual raw message of the notice to be displayed
    pub msg: String,
    ///Is this a notice about a source, if so, this will be Some(source)
    pub source: Option<NoticeSource>,
    ///The level of the notice which determines how to print it.
    pub level: NoticeLevel,
    ///A child notice is a notice that is part of a series of notices. This is so that they can be preorderly printed together
    pub children: Vec<Notice>
}

impl Notice {
    pub fn new(from: String, msg: String, file: Option<String>, pos: Option<BiPos>, level: NoticeLevel, children: Vec<Notice>) -> Self{
        Self{
            from,
            msg,
            source: if let Some(file) = file{
                if let Some(pos) = pos{
                    Some(NoticeSource{
                        file,
                        pos
                    })
                }else{
                    None
                }
            }else{
                None
            },
            level,
            children
        }
    }

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

        
        if self.source.is_none(){
            return;
        }

        let _source = self.source.unwrap();
        let file = _source.file;
        let pos = _source.pos;
    
        println!("\tat [{}:({},{}) to ({},{})]\n", file, pos.start.0 + 1, pos.start.1 + 1, pos.end.0 + 1, pos.end.1 + 1);
    
        if let Some(src) = source {
            if let Some((start_line, lines, squiggly)) = pos.locate_in_source(src) {
                lines.iter().enumerate().for_each(|(i, line)| {
                    println!(
                        "\t{}{:4}{} | {}",
                        colour,
                        start_line + i + 1,
                        ansi::Fg::Reset,
                        line
                    );

                    if i == pos.start.0 {
                        println!("\t{}---- | {}{}", colour, squiggly, ansi::Fg::Reset);
                    };
                });
                println!();
            }
        }

    }
}

pub const TAB_WIDTH: usize = 5;
const ERR_LINE_OFFSET: usize = 2;

pub trait SourceOrigin {
    fn locate_in_source(self, source: &str) -> Option<(usize, Vec<&str>, String)>;
}

impl SourceOrigin for BiPos {
    fn locate_in_source(self, source: &str) -> Option<(usize, Vec<&str>, String)> {
        let start_line = if self.start.0 >= ERR_LINE_OFFSET {
            self.start.0 - ERR_LINE_OFFSET
        } else {
            0
        };
        let lines: Vec<&str> = source
            .lines()
            .skip(start_line as usize)
            .take(7)
            .collect();
        let error_line = if lines.len() > ERR_LINE_OFFSET {
            lines[ERR_LINE_OFFSET]
        } else {
            lines[self.start.0 as usize]
        };
        let squiggly_amount = if self.end.1 - self.start.1 == 0{
            1
        }else{
            self.end.1 - self.start.1
        };

        let squiggly = if (self.start.1 as usize - 1) < error_line.len() {
            core::padding::padding("^", squiggly_amount).to_string()
        } else {
            String::new()
        };
        
        let squiggly_line = format!(
            "{}{}",
            core::padding::padding_until(" ", self.start.1),
            squiggly
        );
        Some((start_line as usize, lines, squiggly_line))
    }
}
