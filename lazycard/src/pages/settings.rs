use std::str::FromStr;

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use widgets::{CursorMove, List, ListItem, Shortcut, Shortcuts, TextInput};

use crate::{
    app::{Action, AppInput, AppRender},
    pages::Log,
    settings::{Colors, Settings},
    symbols,
};

pub struct SettingsPage {
    default: Settings,
    saved: Settings,
    saved_hash: u64,
    is_saved: bool,
    list: List,
    primary: ColorSetting,
    secondary: ColorSetting,
    neutral: ColorSetting,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Setting {
    General,
    DesiredRetention,
    Colors,
    PrimaryColor,
    SecondaryColor,
    NeutralColor,
    Empty,
}

impl Setting {
    const fn filter(&self) -> bool {
        match self {
            Self::General => false,
            Self::DesiredRetention => true,
            Self::Colors => false,
            Self::PrimaryColor => true,
            Self::SecondaryColor => true,
            Self::NeutralColor => true,
            Self::Empty => false,
        }
    }
}

const SETTINGS: [Setting; 9] = [
    Setting::General,
    Setting::Empty,
    Setting::DesiredRetention,
    Setting::Empty,
    Setting::Colors,
    Setting::Empty,
    Setting::PrimaryColor,
    Setting::SecondaryColor,
    Setting::NeutralColor,
];

impl SettingsPage {
    pub fn new(settings: &Settings) -> Self {
        let colors = settings.colors();
        let hash = settings.hash();
        let selected = if SETTINGS[0].filter() {
            0
        } else {
            next(0).unwrap()
        };

        Self {
            default: Settings::default(),
            saved: settings.clone(),
            saved_hash: hash,
            is_saved: true,
            list: List::new().with_index(selected).with_margins(3, 3),
            primary: ColorSetting::new(colors.primary),
            secondary: ColorSetting::new(colors.secondary),
            neutral: ColorSetting::new(colors.neutral),
        }
    }

    pub fn on_enter(&self) {}

    pub fn on_render(&mut self, render: AppRender, settings: &Settings, shortcuts: &mut Shortcuts) {
        let area = render.area().centered_horizontally(Constraint::Max(80));
        let buf = render.buffer();
        let colors = settings.colors();

        let mut last_y = area.y;
        let current_setting = self.current();

        self.list.set_scrollbar(colors.scrollbar()).render(
            area,
            buf,
            SETTINGS,
            |line, buf, setting, index| {
                last_y = line.y;

                let setting_area = Rect {
                    width: line.width / 2,
                    ..line
                };
                let input_area = Rect {
                    x: setting_area.x + setting_area.width + 1,
                    width: setting_area.width.saturating_sub(1),
                    ..setting_area
                };

                let (symbol, style) = if index == ListItem::Selected {
                    (
                        symbols::concat!(symbols::SELECTED, " "),
                        Style::new().bold(),
                    )
                } else {
                    ("", Style::new())
                };

                match setting {
                    Setting::General => {
                        print_section(line, buf, "GENERAL", colors.neutral);
                    }
                    Setting::DesiredRetention => {
                        print_desired_retention(
                            setting_area,
                            input_area,
                            buf,
                            symbol,
                            "Desired retention",
                            style,
                            settings.desired_retention(),
                        );
                    }
                    Setting::Colors => {
                        print_section(line, buf, "COLORS", colors.neutral);
                    }
                    Setting::PrimaryColor => {
                        print_color(
                            setting_area,
                            input_area,
                            buf,
                            symbol,
                            "Primary color",
                            style,
                            &mut self.primary,
                            current_setting == Setting::PrimaryColor,
                            colors,
                        );
                    }
                    Setting::SecondaryColor => {
                        print_color(
                            setting_area,
                            input_area,
                            buf,
                            symbol,
                            "Secondary color",
                            style,
                            &mut self.secondary,
                            current_setting == Setting::SecondaryColor,
                            colors,
                        );
                    }
                    Setting::NeutralColor => {
                        print_color(
                            setting_area,
                            input_area,
                            buf,
                            symbol,
                            "Neutral color",
                            style,
                            &mut self.neutral,
                            current_setting == Setting::NeutralColor,
                            colors,
                        );
                    }
                    Setting::Empty => {}
                }
            },
        );

        // Description and shortcuts
        const COLOR_DESCRIPTION: &str = "Set color by name, hex code or indexed value";
        let description = match current_setting {
            Setting::DesiredRetention => {
                shortcuts.push(Shortcut::new(
                    "Increment/Decrement",
                    symbols::ARROW_RIGHT_LEFT,
                ));
                "Set desired retention in percent"
            }
            Setting::PrimaryColor | Setting::SecondaryColor | Setting::NeutralColor => {
                shortcuts.push(Shortcut::new("Set color", symbols::ENTER));
                COLOR_DESCRIPTION
            }
            Setting::General | Setting::Colors | Setting::Empty => "",
        };

        if !description.is_empty() {
            let description_area = Rect {
                y: area.y + area.height.saturating_sub(1),
                height: 1,
                ..area
            };
            if description_area.y >= last_y + 2 {
                widgets::print_asciis(
                    description_area,
                    buf,
                    [" ", description, " "],
                    colors.neutral,
                    Some(widgets::Alignment::CenterHorizontal),
                );
            }
        }

        if !self.is_saved {
            shortcuts.push(Shortcut::new("Save", symbols::ctrl!("s")));
        }

        // Always show reset all
        shortcuts.push(Shortcut::new("Reset all", symbols::ctrl!("r")));
    }

    pub fn on_input(&mut self, input: AppInput, settings: &mut Settings) -> Action {
        let (key, modifiers) = input.key_pressed_and_modifiers();
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);

        match key {
            KeyCode::Down => {
                if let Some(next) = next(self.list.index()) {
                    self.list.set_index(next);
                    return Action::Render;
                }
            }
            KeyCode::Up => {
                if let Some(prev) = previous(self.list.index()) {
                    self.list.set_index(prev);
                    return Action::Render;
                }
            }
            KeyCode::Char('s') => {
                if ctrl && !self.is_saved {
                    match settings.save() {
                        Ok(_) => {
                            self.saved = settings.clone();
                            self.saved_hash = settings.hash();
                            self.is_saved = true;
                            return Action::Render;
                        }
                        Err(err) => {
                            return Action::EnqueueLog(Log::new(err));
                        }
                    }
                } else {
                    return self.handle_setting(key, modifiers, settings);
                }
            }
            KeyCode::Char('r') => {
                if ctrl {
                    *settings = self.default.clone();
                    self.primary.reset_with(settings.primary());
                    self.secondary.reset_with(settings.secondary());
                    self.neutral.reset_with(settings.neutral());
                    self.update_is_saved(settings);
                    return Action::ApplySettings;
                } else {
                    return self.handle_setting(key, modifiers, settings);
                }
            }
            _ => return self.handle_setting(key, modifiers, settings),
        }

        Action::None
    }

    pub fn on_exit(&self) {}

    fn handle_setting(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        settings: &mut Settings,
    ) -> Action {
        match self.current() {
            Setting::DesiredRetention => {
                if let KeyCode::Left | KeyCode::Right = key {
                    let increment = key == KeyCode::Right;
                    let current_retention = settings.desired_retention();
                    let new_retention = if increment {
                        (current_retention + 1).min(100)
                    } else {
                        current_retention.saturating_sub(1)
                    };
                    if current_retention != new_retention {
                        settings.set_desired_retention(new_retention);
                        self.update_is_saved(settings);
                        return Action::ApplySettings;
                    }
                }
            }
            Setting::PrimaryColor => {
                if let KeyCode::Enter = key {
                    match self.primary.parse_color() {
                        Ok(color) => {
                            if settings.primary() != color {
                                settings.set_primary(color);
                                self.update_is_saved(settings);
                                return Action::Render;
                            }
                        }
                        Err(err) => {
                            let log = Log::new(err);
                            return Action::EnqueueLog(log);
                        }
                    }
                } else if self.primary.input(key, modifiers) {
                    return Action::Render;
                }
            }
            Setting::SecondaryColor => {
                if let KeyCode::Enter = key {
                    match self.secondary.parse_color() {
                        Ok(color) => {
                            if settings.secondary() != color {
                                settings.set_secondary(color);
                                self.update_is_saved(settings);
                                return Action::Render;
                            }
                        }
                        Err(err) => {
                            let log = Log::new(err);
                            return Action::EnqueueLog(log);
                        }
                    }
                } else if self.secondary.input(key, modifiers) {
                    return Action::Render;
                }
            }
            Setting::NeutralColor => {
                if let KeyCode::Enter = key {
                    match self.neutral.parse_color() {
                        Ok(color) => {
                            if settings.neutral() != color {
                                settings.set_neutral(color);
                                self.update_is_saved(settings);
                                return Action::Render;
                            }
                        }
                        Err(err) => {
                            let log = Log::new(err);
                            return Action::EnqueueLog(log);
                        }
                    }
                } else if self.neutral.input(key, modifiers) {
                    return Action::Render;
                }
            }
            Setting::General | Setting::Colors | Setting::Empty => {}
        }

        Action::None
    }

    fn update_is_saved(&mut self, settings: &Settings) {
        self.is_saved = self.saved_hash == settings.hash();
    }

    const fn current(&self) -> Setting {
        SETTINGS[self.list.index()]
    }
}

struct ColorSetting(TextInput);

impl ColorSetting {
    fn new(color: Color) -> Self {
        let mut input = TextInput::from(color.to_string());
        input.move_cursor(CursorMove::End, false);
        Self(input)
    }

    fn parse_color(&self) -> Result<Color, String> {
        let input = self.0.as_str_trim();
        Color::from_str(input).map_err(|_| format!("Failed to parse \"{input}\" as a color"))
    }

    const fn set_active(&mut self, active: bool, colors: &Colors) {
        self.0.set_disabled(!active).set_colors(colors.text_input());
    }

    fn input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        self.0.input(key, modifiers)
    }

    fn render(&mut self, line: Rect, buf: &mut Buffer) {
        self.0.render(line, buf);
    }

    fn reset_with(&mut self, color: Color) {
        self.0.clear();
        self.0.push_str(color.to_string().as_str());
    }
}

fn next(current: usize) -> Option<usize> {
    let mut next = current + 1;
    while next < SETTINGS.len() {
        if SETTINGS[next].filter() {
            return Some(next);
        }
        next += 1;
    }

    None
}

fn previous(current: usize) -> Option<usize> {
    if current == 0 {
        return None;
    }

    let mut prev = current.saturating_sub(1);
    loop {
        if SETTINGS[prev].filter() {
            return Some(prev);
        }

        if prev == 0 {
            break;
        }

        prev -= 1;
    }

    None
}

fn print_section(line: Rect, buf: &mut Buffer, ascii: &str, color: Color) {
    widgets::print_ascii(
        line,
        buf,
        ascii,
        color,
        Some(widgets::Alignment::CenterHorizontal),
    );
}

fn print_desired_retention(
    text_area: Rect,
    input_area: Rect,
    buf: &mut Buffer,
    symbol: &str,
    text: &str,
    style: Style,
    desired_retention: u8,
) {
    // Text
    widgets::print_asciis(
        text_area,
        buf,
        [symbol, text, ":"],
        style,
        Some(widgets::Alignment::Right),
    );

    // Desired retention
    utils::format_int(desired_retention, |desired_retention| {
        widgets::print_asciis(
            input_area,
            buf,
            [desired_retention, "%"],
            Style::new(),
            None,
        );
    });
}

fn print_color(
    text_area: Rect,
    input_area: Rect,
    buf: &mut Buffer,
    symbol: &str,
    text: &str,
    style: Style,
    color_setting: &mut ColorSetting,
    color_is_active: bool,
    colors: &Colors,
) {
    // Text
    widgets::print_asciis(
        text_area,
        buf,
        [symbol, text, ":"],
        style,
        Some(widgets::Alignment::Right),
    );

    // Input
    color_setting.set_active(color_is_active, colors);
    color_setting.render(input_area, buf);
}
