use std::path::PathBuf;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use widgets::{ScrollbarColors, SyntaxHighlightTheme, TextEditorColors, TextInputColors};

const VERSION: u8 = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub general: General,
    pub colors: Colors,

    #[serde(skip)]
    theme: ThemeMode,
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        let general = General {
            desired_retention: 80,
        };
        let theme = ThemeMode::default();
        let colors = Colors::from_theme(theme);

        Self {
            general,
            colors,
            theme,
            path: None,
        }
    }
}

impl Settings {
    pub const fn desired_retention(&self) -> u8 {
        self.general.desired_retention
    }

    pub const fn desired_retention_as_fraction(&self) -> f32 {
        self.general.desired_retention as f32 / 100.0
    }

    pub const fn set_desired_retention(&mut self, percent: u8) {
        self.general.desired_retention = if percent > 100 { 100 } else { percent };
    }

    pub const fn colors(&self) -> &Colors {
        &self.colors
    }

    pub const fn primary(&self) -> Color {
        self.colors.primary
    }

    pub const fn secondary(&self) -> Color {
        self.colors.secondary
    }

    pub const fn neutral(&self) -> Color {
        self.colors.neutral
    }

    pub const fn syntax_highlight_theme(&self) -> SyntaxHighlightTheme {
        match self.theme {
            ThemeMode::Dark => SyntaxHighlightTheme::Base16EightiesDark,
            ThemeMode::Light => SyntaxHighlightTheme::InspiredGitHub,
        }
    }

    pub const fn set_primary(&mut self, color: Color) {
        self.colors.primary = color;
    }

    pub const fn set_secondary(&mut self, color: Color) {
        self.colors.secondary = color;
    }

    pub const fn set_neutral(&mut self, color: Color) {
        self.colors.neutral = color;
    }

    pub fn read(path: Option<PathBuf>) -> Result<Self, String> {
        let Some(file) = path.or_else(|| get_config_file()) else {
            return Ok(Self::default());
        };

        let bytes = match std::fs::read(&file) {
            Ok(bytes) => bytes,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    return Ok(Self::default().with_path(Some(file)));
                }
                _ => {
                    return Err(format!(
                        "Failed to read settings from \"{}\" due to {}",
                        file.display(),
                        err
                    ))?;
                }
            },
        };

        #[derive(Deserialize)]
        struct V {
            version: u8,
        }

        let V { version } = toml::from_slice(&bytes).map_err(|err| {
            format!(
                "Failed to deserialize settings from \"{}\" due to {}",
                file.display(),
                err
            )
        })?;

        let mut settings: Self = match version {
            VERSION => toml::from_slice(&bytes).map_err(|err| {
                format!(
                    "Failed to deserialize settings from \"{}\" due to {}",
                    file.display(),
                    err
                )
            })?,
            _ => Err(format!(
                "Failed to deserialize settings from \"{}\" due to unknown version",
                file.display()
            ))?,
        };

        settings.path = Some(file);
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(file) = self.path.clone().or_else(|| get_config_file()) else {
            return Err("Unable to save settings due to no path specified or \
                a default one could not be retrieved from the operating system")?;
        };

        if let Some(parent) = file.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(&parent).map_err(|err| {
                    format!(
                        "Unable to save settings as creating directory \"{}\" failed due to {}",
                        parent.display(),
                        err
                    )
                })?;
            }
        }

        let toml = toml::to_string(self)
            .map_err(|err| format!("Failed to serialize settings due to {}", err))?;
        std::fs::write(&file, toml).map_err(|err| {
            format!(
                "Failed to write settings to \"{}\" due to {}",
                file.display(),
                err
            )
        })?;

        Ok(())
    }

    pub fn hash(&self) -> u64 {
        toml::to_string(self)
            .map(|s| utils::hash_fast(s))
            .unwrap_or(0)
    }

    pub(super) fn with_path(mut self, path: Option<PathBuf>) -> Self {
        self.path = path;
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct General {
    desired_retention: u8,
}

#[derive(Debug, Clone, Copy)]
enum ThemeMode {
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        match terminal_colorsaurus::theme_mode(terminal_colorsaurus::QueryOptions::default())
            .unwrap_or(terminal_colorsaurus::ThemeMode::Dark)
        {
            terminal_colorsaurus::ThemeMode::Dark => Self::Dark,
            terminal_colorsaurus::ThemeMode::Light => Self::Light,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Colors {
    pub primary: Color,
    pub secondary: Color,
    pub neutral: Color,
}

impl Colors {
    const fn from_theme(theme: ThemeMode) -> Self {
        let neutral = Color::Indexed(245);
        match theme {
            ThemeMode::Dark => Self {
                primary: Color::LightYellow,
                secondary: Color::Yellow,
                neutral,
            },
            ThemeMode::Light => Self {
                primary: Color::LightCyan,
                secondary: Color::Cyan,
                neutral,
            },
        }
    }

    pub const fn text_input(&self) -> TextInputColors {
        TextInputColors {
            normal: Color::Reset,
            cursor: self.secondary,
            selector: self.neutral,
            placeholder: self.neutral,
            disabled: self.neutral,
        }
    }

    pub const fn text_editor(&self) -> TextEditorColors {
        TextEditorColors {
            normal: Color::Reset,
            cursor: self.secondary,
            selector: self.neutral,
            placeholder: self.neutral,
            disabled: self.neutral,
        }
    }

    pub const fn scrollbar(&self) -> ScrollbarColors {
        ScrollbarColors {
            thumb: self.neutral,
            track: None,
        }
    }
}

fn get_config_file() -> Option<PathBuf> {
    const FILENAME: &str = "settings.toml";
    directories::ProjectDirs::from(
        crate::APP_QUALIFIER,
        crate::APP_ORGANIZATION,
        crate::APP_NAME,
    )
    .map(|project_dirs| project_dirs.config_dir().join(FILENAME))
}
