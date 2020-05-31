use annotate_snippets::snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation};
use core::pos::BiPos;

pub struct SnippetBuilder {
    snippet: Snippet,
}

impl Default for SnippetBuilder {
    fn default() -> Self {
        SnippetBuilder {
            snippet: Snippet {
                title: None,
                footer: vec![],
                slices: vec![],
            },
        }
    }
}

impl SnippetBuilder {
    pub fn new() -> SnippetBuilder {
        SnippetBuilder::default()
    }
    pub fn title(mut self, title: Annotation) -> SnippetBuilder {
        self.snippet.title = Some(title);
        self
    }
    pub fn footer(mut self, footer: Annotation) -> SnippetBuilder {
        self.snippet.footer.push(footer);
        self
    }
    pub fn slice(&mut self, slice: Slice) {
        self.snippet.slices.push(slice);
    }
    pub fn build(self) -> Snippet {
        self.snippet
    }
}

pub struct SliceBuilder {
    slice: Slice,
}

impl SliceBuilder {
    pub fn new(fold: bool) -> SliceBuilder {
        SliceBuilder {
            slice: Slice {
                source: String::new(),
                line_start: 0,
                origin: None,
                annotations: Vec::new(),
                fold,
            },
        }
    }

    pub fn origin(mut self, relative_file_path: &str) -> SliceBuilder {
        self.slice.origin = Some(relative_file_path.to_string());
        self
    }

    pub fn line_start(mut self, line_start: usize) -> Self {
        self.slice.line_start = line_start;
        self
    }

    pub fn source(mut self, source: String) -> Self{
        self.slice.source = source;
        self
    }

    pub fn source_annotation(
        &mut self,
        range: (usize, usize),
        label: &str,
        source_annotation_type: AnnotationType,
    ) {
        self.slice.annotations.push(SourceAnnotation {
            range,
            label: label.to_string(),
            annotation_type: source_annotation_type,
        });
    }

    pub fn build(mut self) -> Slice {
        self.slice
    }
}

pub struct AnnotationBuilder {
    annotation: Annotation,
}

impl AnnotationBuilder {
    pub fn new(annotation_type: AnnotationType) -> AnnotationBuilder {
        AnnotationBuilder {
            annotation: Annotation {
                id: None,
                label: None,
                annotation_type,
            },
        }
    }

    pub fn id(mut self, id: &str) -> AnnotationBuilder {
        self.annotation.id = Some(id.to_string());
        self
    }

    pub fn label(mut self, label: &str) -> AnnotationBuilder {
        self.annotation.label = Some(label.to_string());
        self
    }

    pub fn build(self) -> Annotation {
        self.annotation
    }
}