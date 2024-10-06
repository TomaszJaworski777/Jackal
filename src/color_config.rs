use colored::Colorize;

pub struct ColorConfig;
impl ColorConfig {
    pub const WIN_COLOR: (u8,u8,u8) = (80, 191, 71);
    pub const DRAW_COLOR: (u8,u8,u8) = (225, 225, 120);
    pub const LOSE_COLOR: (u8,u8,u8) = (200, 71, 71);

    pub const LABEL_COLOR: (u8,u8,u8) = (191,191,191);
    pub const HIGHLIGHT_COLOR: (u8,u8,u8) = (133,209,227);
    pub const HIGHLIGHT_ALT_COLOR: (u8,u8,u8) = (225,225,225);
}

pub trait Colored {
    fn label(&self) -> String;
    fn highlight(&self) -> String;
    fn highlight_alt(&self) -> String;
}

impl Colored for String {
    fn label(&self) -> Self { 
        self.truecolor(ColorConfig::LABEL_COLOR.0, ColorConfig::LABEL_COLOR.1, ColorConfig::LABEL_COLOR.2).to_string()
    }
    fn highlight(&self) -> Self {
        self.truecolor(ColorConfig::HIGHLIGHT_COLOR.0, ColorConfig::HIGHLIGHT_COLOR.1, ColorConfig::HIGHLIGHT_COLOR.2).to_string()
    }
    fn highlight_alt(&self) -> Self {
        self.truecolor(ColorConfig::HIGHLIGHT_ALT_COLOR.0, ColorConfig::HIGHLIGHT_ALT_COLOR.1, ColorConfig::HIGHLIGHT_ALT_COLOR.2).bold().to_string()
    }
}

impl Colored for &str {
    fn label(&self) -> String { 
        self.truecolor(ColorConfig::LABEL_COLOR.0, ColorConfig::LABEL_COLOR.1, ColorConfig::LABEL_COLOR.2).to_string()
    }
    fn highlight(&self) -> String {
        self.truecolor(ColorConfig::HIGHLIGHT_COLOR.0, ColorConfig::HIGHLIGHT_COLOR.1, ColorConfig::HIGHLIGHT_COLOR.2).to_string()
    }
    fn highlight_alt(&self) -> String {
        self.truecolor(ColorConfig::HIGHLIGHT_ALT_COLOR.0, ColorConfig::HIGHLIGHT_ALT_COLOR.1, ColorConfig::HIGHLIGHT_ALT_COLOR.2).bold().to_string()
    }
}