// ----------
// ANSI CODES
// ----------

/// ANSI escape codes for text formatting
#[derive(Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum ANSI {
    // Style codes
    Reset = 0,
    Bold,
    Faint,
    Italic,
    Underline,
    BlinkSlow,
    BlinkRapid,
    Reverse,
    Conceal,
    CrossedOut = 9,
    NormalIntensity = 22,
    NotItalic,
    NotUnderline,
    NotBlink,
    NotReverse,
    NotConceal,
    NotCrossedOut = 29,

    // Foreground colors
    Black = 30,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White = 37,
    Default = 39,

    // Background colors
    BgBlack = 40,
    BgRed,
    BgGreen,
    BgYellow,
    BgBlue,
    BgMagenta,
    BgCyan,
    BgWhite,
    BgDefault = 49,

    // Bright foreground colors
    BrightBlack = 90,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    BrightDefault = 99,

    // Bright background colors
    BgBrightBlack = 100,
    BgBrightRed,
    BgBrightGreen,
    BgBrightYellow,
    BgBrightBlue,
    BgBrightMagenta,
    BgBrightCyan,
    BgBrightWhite,
    BgBrightDefault = 109,
}

impl std::fmt::Display for ANSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

pub trait ANSIString {
    fn ansi(self, codes: &[ANSI]) -> String;
}

impl ANSIString for &str {
    fn ansi(self, codes: &[ANSI]) -> String {
        let codes_str = codes
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(";");
        format!("\u{001b}[{}m{}\u{001b}[0m", codes_str, self)
    }
}

impl ANSIString for String {
    fn ansi(self, codes: &[ANSI]) -> String {
        let codes_str = codes
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(";");
        format!("\u{001b}[{}m{}\u{001b}[0m", codes_str, &self)
    }
}
