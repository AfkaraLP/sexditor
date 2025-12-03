pub mod cursor_actions;
pub mod text_actions;
pub mod text_colour;

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use std::fs::read_to_string;
use std::io::Write;

use crate::editor;
use crate::editor::cursor_actions::CursorAction;
use crate::editor::text_actions::TextAction;

use crate::{
    editor::text_colour::{RUST_SYNTAX, SyntaxRegex, colour_text},
    theme::ColourTheme,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use fancy_regex::Regex;
use ratatui::{
    DefaultTerminal, Frame,
    widgets::{Block, BorderType, Paragraph},
};

#[derive(Debug, Copy, Clone)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default, Debug)]
pub struct Editor {
    pub cursor: Position,
    pub mode: EditorMode,
    pub file_text: String,
    pub file_path: String,
    pub keyhistory: Vec<KeyCode>,
    pub exit: bool,
    pub command: String,
    pub frame_area: Rect,
    pub scroll: Position,
    pub theme_path: String,
    pub message_queue: LogMessage,
}
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
pub enum EditorMode {
    #[default]
    Normal,
    Visual,
    Insert,
    Command,
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Editor {
    pub fn new(path: Option<String>) -> Self {
        let mut res = Self::default();
        res.open_new_file(path);
        res
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            self.frame_area = terminal.get_frame().area();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.set_cursor_position((self.cursor.x + 1, self.cursor.y + 1));

        frame.render_stateful_widget(self, frame.area(), &mut State);
    }

    pub fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            Event::Resize(x, y) => self.handle_resize(x, y),
            _ => {}
        }
        Ok(())
    }
    pub fn handle_resize(&mut self, x: u16, y: u16) {
        self.frame_area = Rect {
            x: 0,
            y: 0,
            width: x,
            height: y,
        }
    }

    /// Opens `[scratch]` buffer if no path is provided
    pub fn open_new_file(&mut self, path: Option<String>) {
        let Some(path) = path else {
            self.file_text = String::new();
            self.file_path = "[scratch]".into();
            return;
        };
        self.file_path.clone_from(&path);
        match read_to_string(path) {
            Ok(content) => {
                self.file_text = content;
            }
            Err(_) => self.file_text = String::new(),
        }
    }

    pub fn save_file(&self) {
        let mut file =
            std::fs::File::create(self.file_path.as_str()).expect("directory does not exist");
        file.write_all(self.file_text.as_bytes())
            .expect("failed to write to file");
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.mode {
            EditorMode::Normal => {
                if let KeyCode::Char(c) = key_event.code {
                    match c {
                        'q' => self.exit(),
                        'i' => self.mode = EditorMode::Insert,
                        'v' => self.mode = EditorMode::Visual,
                        ':' => self.mode = EditorMode::Command,
                        'k' => self.move_cursor(CursorDirection::Up),
                        'j' => self.move_cursor(CursorDirection::Down),
                        'h' => self.move_cursor(CursorDirection::Left),
                        'l' => self.move_cursor(CursorDirection::Right),
                        'd' => self.remove_char(self.cursor),
                        'o' => {
                            self.insert_char(
                                Position {
                                    x: u16::try_from(self.line_at_cursor().len())
                                        .unwrap_or_default()
                                        + 1,
                                    y: self.cursor.y,
                                },
                                '\n',
                            );
                            self.move_cursor(CursorDirection::Down);
                            self.mode = EditorMode::Insert;
                        }
                        'O' => {
                            self.insert_char(
                                Position {
                                    x: u16::try_from(self.line_from_cursor(-1).len())
                                        .unwrap_or_default()
                                        + 1,
                                    y: self.cursor.y - 1,
                                },
                                '\n',
                            );
                            self.mode = EditorMode::Insert;
                        }
                        'A' => {
                            self.cursor.x =
                                u16::try_from(self.line_at_cursor().len()).unwrap_or_default();
                            self.mode = EditorMode::Insert;
                        }
                        '0' => self.cursor.x = 0,
                        'e' => self.move_to_end_of_pat(
                            &Regex::new(r"(\p{Z}+|\p{P}+|\p{N}+|\p{L}+|\p{S}+)").unwrap(),
                        ),
                        'b' => self.move_to_start_of_pat(
                            &Regex::new(r"(\p{Z}+|\p{P}+|\p{N}+|\p{L}+|\p{S}+)").unwrap(),
                        ),
                        'g' => {
                            if let Some(KeyCode::Char('g')) = self.keyhistory.last() {
                                self.cursor = Position::default();
                            }
                        }
                        _ => {}
                    }
                }
            }
            EditorMode::Visual => match key_event.code {
                KeyCode::Char('v') | KeyCode::Esc => self.mode = EditorMode::Normal,
                _ => {}
            },
            EditorMode::Insert => match key_event.code {
                KeyCode::Char(c) => {
                    self.insert_char(self.cursor, c);
                    self.move_cursor(CursorDirection::Right);
                }
                KeyCode::Enter => {
                    self.insert_char(self.cursor, '\n');
                    self.cursor = Position {
                        x: 0,
                        y: self.cursor.y + 1,
                    }
                }
                KeyCode::Backspace => {
                    self.remove_char(Position {
                        x: self.cursor.x - 1,
                        y: self.cursor.y,
                    });
                    self.move_cursor(CursorDirection::Left);
                }
                KeyCode::Esc => self.mode = EditorMode::Normal,
                _ => {}
            },
            EditorMode::Command => match key_event.code {
                KeyCode::Enter => self.execute_command(),
                KeyCode::Esc => self.end_command(),
                KeyCode::Char(c) => self.command.push(c),
                KeyCode::Backspace => _ = self.command.pop(),
                _ => {}
            },
        }
        self.keyhistory.push(key_event.code);
    }
    pub fn execute_command(&mut self) {
        match self.command.clone().trim() {
            "q" => self.exit(),
            "w" => self.save_file(),
            "x" => {
                self.save_file();
                self.exit();
            }
            "e" => self.log(LogMessage::Error("aaaa".into())),
            path if path.starts_with("theme ") => {
                self.set_theme(Some(&path["theme ".len()..]));
            }
            _ => {}
        }
        self.end_command();
    }
    pub fn end_command(&mut self) {
        self.mode = EditorMode::Normal;
        self.command = String::new();
    }
    pub fn set_theme(&mut self, path: Option<impl ToString>) {
        let path = path.map_or("default".to_string(), |v| v.to_string());
        let full_path = ["theme", &path].join("/");
        let full_path = [full_path, "toml".into()].join(".");
        self.theme_path = full_path;
    }
    pub fn log(&mut self, msg: LogMessage) {
        self.message_queue = msg;
    }
}

#[derive(Debug)]
pub enum LogMessage {
    Error(String),
    Warn(String),
    Info(String),
}

impl Default for LogMessage {
    fn default() -> Self {
        LogMessage::Info(String::new())
    }
}

impl LogMessage {
    pub fn to_paragraph(&self) -> Paragraph<'_> {
        match self {
            LogMessage::Error(msg) => {
                Paragraph::new(msg.as_str()).style(Style::new().fg(Color::Red))
            }

            LogMessage::Warn(msg) => {
                Paragraph::new(msg.as_str()).style(Style::new().fg(Color::Yellow))
            }
            LogMessage::Info(msg) => {
                Paragraph::new(msg.as_str()).style(Style::new().fg(Color::White))
            }
        }
    }
}

pub struct State;

impl StatefulWidget for &Editor {
    type State = State;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _state: &mut editor::State,
    ) where
        Self: Sized,
    {
        let theme = read_to_string(self.theme_path.as_str())
            .unwrap_or(include_str!("../../theme/default.toml").to_string());
        let theme: ColourTheme = toml::from_str(&theme).unwrap();

        let syntax_lang = self.file_path.split('.').next_back().unwrap_or_default();
        let syntax_path = format!("./syntax/{syntax_lang}.toml");
        let syntax = read_to_string(syntax_path);
        let syntax: SyntaxRegex = syntax
            .map(|syntax| toml::from_str::<SyntaxRegex>(&syntax).unwrap_or(RUST_SYNTAX.clone()))
            .unwrap_or(RUST_SYNTAX.clone());

        let title = Line::from(self.file_path.as_str());
        let mode = Line::from(format!("{:#?}", self.mode));
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title.centered())
            .title_bottom(mode.left_aligned())
            .style(Style::new().bg(theme.background.into()))
            .border_set(border::THICK);
        let text = self.file_text.as_str();
        let text = colour_text(text, &theme, &syntax);

        let adjusted_area = area;

        let scroll_height = (self.cursor.y + 1).saturating_sub(adjusted_area.height);

        self.message_queue.to_paragraph().render(
            Rect {
                x: self.cursor.x,
                y: self.cursor.y + 2,
                width: 20,
                height: 20,
            },
            buf,
        );

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .scroll((scroll_height, 0))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .render(adjusted_area, buf);

        if self.mode == EditorMode::Command {
            let command_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("Command")
                .style(Style::new().fg(Color::White).bg(theme.background.into()));

            let percent_80: u16 = (f32::from(adjusted_area.width) * 0.8).round() as u16;
            let percent_10: u16 = (f32::from(adjusted_area.width) * 0.1).round() as u16;
            Paragraph::new(self.command.as_str())
                .style(Style::new().fg(Color::White).bg(theme.background.into()))
                .block(command_block)
                .left_aligned()
                .render(
                    Rect::new(
                        adjusted_area.x + percent_10,
                        adjusted_area.y + 4,
                        percent_80,
                        3,
                    ),
                    buf,
                );
        }
    }
}
