use ratatui::style::Color;

// TODO: general and version

pub struct Settings {
    pub general: General,
    pub colors: Colors,
}

impl Default for Settings {
    fn default() -> Self {
        let general = General {};
        let colors =
            match terminal_colorsaurus::theme_mode(terminal_colorsaurus::QueryOptions::default())
                .unwrap_or(terminal_colorsaurus::ThemeMode::Dark)
            {
                terminal_colorsaurus::ThemeMode::Dark => Colors {
                    primary: Color::LightYellow,
                    neutral: Color::Indexed(245),
                    syntax_highlighting: "base16-eighties.dark",
                },
                terminal_colorsaurus::ThemeMode::Light => Colors {
                    primary: Color::LightCyan,
                    neutral: Color::Indexed(245),
                    syntax_highlighting: "InspiredGitHub",
                },
            };

        Self { general, colors }
    }
}

impl Settings {
    pub const fn colors(&self) -> &Colors {
        &self.colors
    }

    pub const fn primary(&self) -> Color {
        self.colors.primary
    }

    pub const fn neutral(&self) -> Color {
        self.colors.neutral
    }

    pub const fn syntax_highlighting(&self) -> &'static str {
        self.colors.syntax_highlighting
    }

    pub const fn set_primary(&mut self, color: Color) {
        self.colors.primary = color;
    }

    pub const fn set_neutral(&mut self, color: Color) {
        self.colors.neutral = color;
    }
}

pub struct General {}

pub struct Colors {
    pub primary: Color,
    pub neutral: Color,
    pub syntax_highlighting: &'static str,
}
