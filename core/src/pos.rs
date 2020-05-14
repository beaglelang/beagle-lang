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
        }
    }
}

impl BiPos {
    pub fn next_line(&mut self) {
        self.start.0 += 1;
        self.start.1 = 0;
        self.end = self.start;
    }

    pub fn next_col(&mut self) {
        self.start = self.end;
        self.start.1 += 1;
        self.end.1 += 1;
    }

    pub fn next_col_end(&mut self) {
        self.end.1 += 1;
    }
}
