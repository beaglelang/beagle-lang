#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Fg{
    Black = 30,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset = 39,
    BrightBlack = 90,
    BrightRed,
    BrightGreen,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl std::fmt::Display for Fg{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "\x1b[{}m", *self as u8)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Bg{
    Black = 40,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset = 49,
    BrightBlack = 100,
    BrightRed,
    BrightGreen,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl std::fmt::Display for Bg{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "\x1b[{}m", *self as u8)
    }
}