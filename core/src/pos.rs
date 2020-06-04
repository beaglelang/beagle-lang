use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position(pub usize, pub usize);
impl Default for Position {
    fn default() -> Self {
        Self(0, 0)
    }
}

// impl std::fmt::Display for Position{
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "start: line {}, col {}\n\tend: line {} col {}",
//             self.start.0, self.start.1, self.end.0, self.end.1
//         )
//     }
// }

///A position consisting of two positions. This is for tracking start and end for complex data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BiPos {
    pub start: Position,
    pub end: Position,
    pub offset: Position,
    ///This is a region in source code where the previous line and the next line are memoized for source snipping purposes
    pub line_region: Position
}

impl std::fmt::Display for BiPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{{{},{}}}, {{{}, {}}}}}", self.start.0 + 1, self.start.1 + 1, self.end.0 + 1, self.end.1 + 1)
    }
}

impl Default for BiPos {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
            offset: Position::default(),
            line_region: Position(0, 2)
        }
    }
}

impl BiPos {
    pub fn next_line(&mut self) {
        self.start.0 += 1;
        self.start.1 = 0;
        self.end = self.start;
        self.offset.1 += 1;
        self.offset.0 = self.offset.1;
        self.line_region.0 += 1;
        self.line_region.1 += 1;
    }

    pub fn next_col(&mut self) {
        self.start = self.end;
        self.start.1 += 1;
        self.end.1 += 1;
        self.offset.0 = self.offset.1;
        self.offset.0 += 1;
        self.offset.1 += 1;
    }

    pub fn next_col_end(&mut self) {
        self.end.1 += 1;
        self.offset.1 += 1;
    }

    pub fn meet(&self, other: &BiPos) -> Self{
        BiPos{
            start: self.start,
            end: other.end,
            offset: Position(self.offset.0, other.offset.1),
            line_region: Position(self.line_region.0, other.line_region.1)
        }
    }

    pub fn range_to(&self, other: &BiPos) -> Self{
        let start = Position(other.start.0 - self.start.0, other.start.1 - self.start.1);
        let end = Position(other.end.0 - self.end.0, other.end.1 - self.end.1);
        BiPos{
            start,
            end,
            offset: Position(self.offset.0, other.offset.1),
            line_region: Position(self.line_region.0, other.line_region.1)
        }
    }

    pub fn col_range(&self) -> (usize, usize){
        (self.start.1, self.end.1)
    }
}
