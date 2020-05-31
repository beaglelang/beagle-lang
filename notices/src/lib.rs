use core::{
    ansi,
    pos::{BiPos},
};


use annotate_snippets::{
    display_list::{
        DisplayList,
    },
    formatter::DisplayListFormatter,
    snippet::{
        AnnotationType,
    },
};

mod annotation_builder;
use annotation_builder::{
    AnnotationBuilder,
    SliceBuilder,
    SnippetBuilder
};

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum NoticeLevel {
    Notice,
    Warning,
    Error,
    Halt,
}

impl NoticeLevel{
    fn to_annotation_type(&self) -> AnnotationType{
        match self{
            NoticeLevel::Notice => AnnotationType::Info,
            NoticeLevel::Warning => AnnotationType::Warning,
            NoticeLevel::Error => AnnotationType::Error,
            _ => return AnnotationType::Info,
        }
    }
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
    pub file: Option<String>,
    pub pos: Option<BiPos>,
    ///The level of the notice which determines how to print it.
    pub level: NoticeLevel,
    ///A child notice is a notice that is part of a series of notices. This is so that they can be preorderly printed together
    pub children: Vec<Notice>
}

impl<'a> Notice {
    pub fn new(from: String, msg: String, file: Option<String>, pos: Option<BiPos>, level: NoticeLevel, children: Vec<Notice>) -> Self{
        Self{
            from,
            msg,
            file,
            pos,
            level,
            children
        }
    }

    pub fn report(self, source: &str) {        
        
        let title = AnnotationBuilder::new(self.level.to_annotation_type()).label(self.msg.as_str()).build();
        let mut snippet = SnippetBuilder::new().title(title);
        let pos = if let Some(pos) = self.pos{
            pos
        }else{
            BiPos::default()
        };
        let file = if let Some(file) = self.file{
            file
        }else{
            String::new()
        };
        for child in self.children{
            let child_pos = child.pos.unwrap();
            let lines = source
                .lines()
                .skip(child_pos.start.0)
                .take(7)
                .collect::<Vec<&str>>()
                .join("\n");
            let msg = child.msg.as_str();
            let mut slice_builder = SliceBuilder::new(false).origin(file.as_str()).line_start(child_pos.start.0).source(lines.to_owned());
            slice_builder.source_annotation(child_pos.col_range(), msg, child.level.to_annotation_type());
            let slice = slice_builder.build();
            snippet.slice(slice);
        }

        let dl = DisplayList::from(snippet.build());
        let dlf = DisplayListFormatter::new(true, false);

        println!("{}", dlf.format(&dl));
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

        let squiggly = if (self.start.1 as usize) < error_line.len() {
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
