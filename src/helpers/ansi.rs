//! This module provides utilities for working with ANSI escape codes.
//!
//! It includes an `ANSI` enum representing various text styles and colors,
//! and an `ANSIString` trait for applying these styles to strings.

/// Represents an ANSI escape code for text formatting.
///
/// Each variant corresponds to a specific SGR (Select Graphic Rendition) code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Ansi {
    // Reset all attributes
    Reset = 0,

    // Styles
    Bold = 1,
    Faint = 2,
    Italic = 3,
    Underline = 4,
    BlinkSlow = 5,
    BlinkRapid = 6,
    Reverse = 7,
    Conceal = 8,
    CrossedOut = 9,

    // Style Resets
    NormalIntensity = 22,
    NotItalic = 23,
    NotUnderline = 24,
    NotBlink = 25,
    NotReverse = 27,
    NotConceal = 28,
    NotCrossedOut = 29,

    // Foreground Colors
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    Default = 39,

    // Background Colors
    BgBlack = 40,
    BgRed = 41,
    BgGreen = 42,
    BgYellow = 43,
    BgBlue = 44,
    BgMagenta = 45,
    BgCyan = 46,
    BgWhite = 47,
    BgDefault = 49,

    // Bright Foreground Colors
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,

    // Bright Background Colors
    BgBrightBlack = 100,
    BgBrightRed = 101,
    BgBrightGreen = 102,
    BgBrightYellow = 103,
    BgBrightBlue = 104,
    BgBrightMagenta = 105,
    BgBrightCyan = 106,
    BgBrightWhite = 107,
}

impl std::fmt::Display for Ansi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// A trait for applying ANSI styling to a string.
pub trait AnsiString {
    /// Wraps the string with the given ANSI codes.
    ///
    /// This method takes a slice of `ANSI` codes, joins them with semicolons,
    /// and formats the string to be displayed with the specified styles.
    fn ansi(&self, codes: &[Ansi]) -> String;
}

impl<T: AsRef<str>> AnsiString for T {
    /// Applies ANSI styling to the string.
    fn ansi(&self, codes: &[Ansi]) -> String {
        let codes_str = codes
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(";");
        format!("\u{001b}[{}m{}\u{001b}[0m", codes_str, self.as_ref())
    }
}
