#[macro_export]
macro_rules! make_color {
    ($color:expr, $text:expr) => {
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    };
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
            eprintln!("{}", make_color!(31, format!($($arg)*)));
    };
}

#[derive(Debug, Copy, Clone)]
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

impl TryFrom<String> for Color {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
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
    pub fn paint(&self, text: &str) -> String {
        make_color!(*self as u8, text)
    }
}
