/// Make color text in terminal
///
/// # Example
///
/// ```rust
/// use cetar::make_color;
///
/// let red = make_color!(31, "Red text");
/// let green = make_color!(32, "Green text");
///
/// println!("{} {}", red, green);
/// ```
///
#[macro_export]
macro_rules! make_color {
    ($color:expr, $text:expr) => {
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    };
}

/// Print text in terminal with red color
///
/// # Example
///
/// ```rust
/// use cetar::{print_error, make_color};
///
/// print_error!("This is an error message");
/// ```
///
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
            eprintln!("{}", make_color!(31, format!($($arg)*)));
    };
}

/// Enum for ANSI color codes
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

impl Default for Color {
    fn default() -> Self {
        Self::White
    }
}

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "black" => Ok(Self::Black),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            _ => Err(anyhow::anyhow!("Invalid color, must be one of: black, red, green, yellow, blue, magenta, cyan, white")),
        }
    }
}

impl Color {
    /// Paint text with color
    ///
    /// # Example
    ///
    /// ```rust
    /// use cetar::color::Color;
    ///
    /// let red = Color::Red.paint("Red text");
    /// let green = Color::Green.paint("Green text");
    ///
    /// println!("{} {}", red, green);
    /// ```
    ///
    pub fn paint(&self, text: &str) -> String {
        make_color!(*self as u8, text)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_color() {
        let red = Color::Red.paint("Red text");
        let green = Color::Green.paint("Green text");

        assert_eq!(red, "\x1b[31mRed text\x1b[0m");
        assert_eq!(green, "\x1b[32mGreen text\x1b[0m");
    }

    #[test]
    fn test_try_from() {
        let table = vec![
            ("black", Color::Black),
            ("red", Color::Red),
            ("green", Color::Green),
            ("yellow", Color::Yellow),
            ("blue", Color::Blue),
            ("magenta", Color::Magenta),
            ("cyan", Color::Cyan),
            ("white", Color::White),
        ];

        for (color, expected) in table {
            let result = Color::try_from(color).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_try_from_invalid() {
        let color = Color::try_from("invalid");
        assert!(color.is_err());
    }

    #[test]
    fn test_make_color() {
        let red = make_color!(31, "Red text");
        let green = make_color!(32, "Green text");

        assert_eq!(red, "\x1b[31mRed text\x1b[0m");
        assert_eq!(green, "\x1b[32mGreen text\x1b[0m");
    }

    #[test]
    fn test_print_error() {
        print_error!("This is an error message");
    }
}
