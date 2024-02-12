use crate::{Config, Style};

pub struct Prompt {
    pub conf: Config,
    last_bg: Option<u8>,
}

impl Prompt {
    pub fn new(conf: Config) -> Self {
        Prompt {
            conf,
            last_bg: None,
        }
    }
}

impl Prompt {
    #[inline]
    fn color_begin(&mut self) {
        print!("\x01\x1b[0");
    }

    #[inline]
    fn color_end(&mut self) {
        print!("m\x02");
    }

    #[inline]
    fn color_fg(&mut self, fg: u8) {
        match fg {
            0..=7 => print!(";{}", 30 + fg),
            8..=15 => print!(";{}", 90 - 8 + fg),
            _ => print!(";38;5;{}", fg),
        }
    }

    #[inline]
    fn color_bg(&mut self, bg: u8) {
        match bg {
            0..=7 => print!(";{}", 40 + bg),
            8..=15 => print!(";{}", 100 - 8 + bg),
            _ => print!(";48;5;{}", bg),
        }
    }

    pub fn style(&mut self, style: Style) {
        self.color_begin();
        self.color_bg(self.last_bg.unwrap());
        self.color_fg(style.color);
        if style.bold {
            print!(";1");
        }
        if style.italic {
            print!(";3");
        }
        if style.underline {
            print!(";4");
        }
        if style.blink {
            print!(";5");
        }
        if style.strike {
            print!(";9");
        }
        self.color_end();
    }

    fn bg(&mut self, bg: u8) {
        self.color_begin();
        self.color_bg(bg);
        self.color_end();
    }
}

impl Prompt {
    pub fn new_part(&mut self, bg: u8) {
        match self.last_bg {
            None => {
                // First part
                self.bg(bg);
                print!(" ");
            }
            Some(last_bg) if last_bg == bg => {
                // Same color part
                self.style(Style::color(0));
                print!(" \u{e0b1} ");
            }
            Some(last_bg) => {
                // Different color part
                print!(" ");
                self.color_begin();
                self.color_fg(last_bg);
                self.color_bg(bg);
                self.color_end();
                print!("\u{e0b0}");
                self.bg(bg);
                print!(" ");
            }
        }
        self.last_bg = Some(bg);
    }

    pub fn finish(&mut self) {
        if self.last_bg.is_some() {
            print!(" ");
            // Reset color
            self.color_begin();
            self.color_end();
        }
    }
}
