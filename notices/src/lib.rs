use core::{
    pos::{BiPos},
};


use annotate_snippets::{
    snippet::{
        Annotation, AnnotationType, Slice, Snippet, SourceAnnotation
    },
    display_list::{
        DisplayList,
    },
    formatter::DisplayListFormatter,
};

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum DiagnosticLevel {
    Note,
    Info,
    Warning,
    Error,
    Halt,
}

impl DiagnosticLevel{
    fn to_annotation_type(&self) -> AnnotationType{
        match self{
            DiagnosticLevel::Note => AnnotationType::Note,
            DiagnosticLevel::Info => AnnotationType::Info,
            DiagnosticLevel::Warning => AnnotationType::Warning,
            DiagnosticLevel::Error => AnnotationType::Error,
            _ => return AnnotationType::Info,
        }
    }
}


pub struct Diagnostic{
    pub msg: String,
    pub level: DiagnosticLevel,
    pub pos: BiPos,
    pub notes: Vec<String>,
    pub sources: Vec<DiagnosticSource>
}

impl Diagnostic{
    fn to_snippet(&self) -> Snippet{
        let mut footer: Vec<Annotation> = vec![];
        for note in self.notes.iter(){
            footer.push(Annotation{
                id: None,
                annotation_type: AnnotationType::Note,
                label: Some(note.clone())
            })
        }
        let mut slices: Vec<Slice> = vec![];
        for source in self.sources.iter(){
            slices.push(source.to_slice());
        }
        Snippet{
            title: Some(Annotation{
                id: None,
                label: Some(self.msg.clone()),
                annotation_type: self.level.to_annotation_type(),
            }),
            footer,
            slices
        }
    }

    pub fn display(&self){
        let dl = DisplayList::from(self.to_snippet());
        let dlf = DisplayListFormatter::new(true, false);
        println!("{}", dlf.format(&dl))
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticSource{
    msg: String,
    line_start: usize,
    range: (usize, usize),
    level: DiagnosticLevel,
    file: String,
    source: String,
}

impl DiagnosticSource{
    pub fn to_slice(&self) -> Slice{
        Slice{
            source: self.source.clone(),
            line_start: self.line_start,
            origin: Some(self.file.clone()),
            annotations: vec![
                SourceAnnotation{
                    range: self.range,
                    annotation_type: self.level.to_annotation_type(),
                    label: self.msg.clone(),
                }
            ],
            fold: false
        }
    }
}

pub struct DiagnosticSourceBuilder{
    diagnostic_source: DiagnosticSource
}

impl DiagnosticSourceBuilder{
    pub fn new(file: String, line_start: usize) -> Self{
        Self{
            diagnostic_source: DiagnosticSource{
                file,
                line_start,
                range: (0, 0),
                level: DiagnosticLevel::Info,
                msg: String::new(),
                source: String::new(),
            }
        }
    }

    pub fn message(mut self, msg: String) -> Self{
        self.diagnostic_source.msg = msg;
        self
    }

    pub fn range(mut self, range: (usize, usize)) -> Self{
        self.diagnostic_source.range = range;
        self
    }

    pub fn source(mut self, source: String) -> Self{
        self.diagnostic_source.source = source;
        self
    }

    pub fn level(mut self, level: DiagnosticLevel) -> Self{
        self.diagnostic_source.level = level;
        self
    }

    pub fn build(self) -> DiagnosticSource{
        self.diagnostic_source
    }
}

pub struct DiagnosticBuilder{
    diagnostic: Diagnostic
}

impl DiagnosticBuilder{
    pub fn new(level: DiagnosticLevel) -> Self{
        Self{
            diagnostic: Diagnostic{
                msg: String::new(),
                pos: BiPos::default(),
                notes: vec![],
                sources: vec![],
                level,
            }
        }
    }

    pub fn message(mut self, msg: String) -> Self{
        self.diagnostic.msg = msg;
        self
    }

    pub fn position(mut self, pos: BiPos) -> Self{
        self.diagnostic.pos = pos;
        self
    }

    pub fn add_note(mut self, note: String) -> Self{
        self.diagnostic.notes.push(note);
        self
    }

    pub fn add_notes(mut self, sources: &[String]) -> Self{
        self.diagnostic.notes.extend(sources.to_vec());
        self
    }

    pub fn add_source(mut self, source: DiagnosticSource) -> Self{
        self.diagnostic.sources.push(source);
        self
    }

    pub fn add_sources(mut self, sources: &[DiagnosticSource]) -> Self{
        self.diagnostic.sources.extend(sources.to_vec());
        self
    }

    pub fn build(self) -> Diagnostic{
        self.diagnostic
    }
}

#[test]
pub fn diagnostic_test(){
    let source = DiagnosticSourceBuilder::new(format!("test.bg"), 1)
        .level(DiagnosticLevel::Error)
        .message(format!("'anotherFunc' takes an argument of type String but found Int"))
        .range((21, 37))
        .source(format!("fun testFunc(){{
    anotherFunc(123)
}}"))
        .build();
    let source2 = DiagnosticSourceBuilder::new(format!("test.bg"), 5)
        .level(DiagnosticLevel::Error)
        .message(format!("'anotherFunc' is declared with parameter 'message' to be type String"))
        .range((16, 31))
        .source(format!("fun anotherFunc(message: String){{

}}"))
        .build();
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message(format!("Type Mismatch"))
        .add_source(source)
        .add_source(source2)
        .build();
    diagnostic.display();
}