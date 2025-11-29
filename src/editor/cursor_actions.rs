use fancy_regex::Regex;

use crate::editor::{CursorDirection, Editor, Position};

pub trait CursorAction {
    fn cursor_at_end_of_file(&self) -> bool;
    fn cursor_at_end_of_line(&self) -> bool;
    fn cursor_at_start_of_file(&self) -> bool;
    fn cursor_at_start_of_line(&self) -> bool;
    fn move_cursor(&mut self, dir: CursorDirection);
    fn move_to_end_of_pat(&mut self, pat: &Regex);
    fn move_to_start_of_pat(&mut self, pat: &Regex);
    fn move_to_next_line(&mut self);
    fn move_to_previous_line(&mut self);
    fn line_at_cursor(&self) -> &str;
    fn line_from_cursor(&self, y: i16) -> &str;
}

impl CursorAction for Editor {
    fn cursor_at_end_of_file(&self) -> bool {
        self.cursor.y as usize >= self.file_text.lines().count() + 1
    }
    fn cursor_at_start_of_file(&self) -> bool {
        self.cursor.y == 0
    }
    fn line_at_cursor(&self) -> &str {
        self.file_text
            .lines()
            .enumerate()
            .find(|(idx, _)| *idx == self.cursor.y as usize)
            .map(|(_, line)| line)
            .unwrap_or_default()
    }
    fn line_from_cursor(&self, y: i16) -> &str {
        self.file_text
            .lines()
            .enumerate()
            .find(|(idx, _)| *idx == self.cursor.y as usize + y as usize)
            .map(|(_, line)| line)
            .unwrap_or_default()
    }
    fn cursor_at_start_of_line(&self) -> bool {
        self.cursor.x == 0
    }
    fn cursor_at_end_of_line(&self) -> bool {
        self.cursor.x as usize >= self.line_at_cursor().chars().count()
    }
    fn move_cursor(&mut self, dir: CursorDirection) {
        match dir {
            CursorDirection::Up => {
                if self.cursor_at_start_of_file() {
                    return ();
                }
                self.cursor.y -= 1;
                let new_line_char_count = self.line_at_cursor().chars().count() as u16;
                if self.cursor.x > new_line_char_count {
                    self.cursor.x = new_line_char_count;
                }
            }
            CursorDirection::Down => {
                if self.cursor_at_end_of_file() {
                    return ();
                }
                self.cursor.y += 1;
                let new_line_char_count = self.line_at_cursor().chars().count() as u16;
                if self.cursor.x > new_line_char_count {
                    self.cursor.x = new_line_char_count;
                }
            }
            CursorDirection::Left => {
                if self.cursor_at_start_of_file() && self.cursor_at_start_of_line() {
                    return ();
                }
                if self.cursor_at_start_of_line() {
                    self.move_to_previous_line();
                } else {
                    self.cursor.x -= 1
                }
            }
            CursorDirection::Right => {
                if self.cursor_at_end_of_line() {
                    self.move_to_next_line();
                } else {
                    self.cursor.x += 1
                }
            }
        }
    }

    fn move_to_end_of_pat(&mut self, pat: &Regex) {
        let line = self.line_at_cursor();

        let start_byte = line
            .char_indices()
            .nth(self.cursor.x as usize)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let slice = &line[start_byte..];

        let mat = match pat.find(slice) {
            Ok(Some(m)) => m,
            _ => return,
        };

        let extra_chars = slice[..mat.end()].chars().count();

        self.cursor.x += extra_chars as u16;
    }
    fn move_to_start_of_pat(&mut self, pat: &Regex) {
        let line = self.line_at_cursor();

        let cursor_byte = line
            .char_indices()
            .nth(self.cursor.x as usize)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let before = &line[..cursor_byte];

        let reversed: String = before.chars().rev().collect();

        let mat = match pat.find(&reversed) {
            Ok(Some(m)) => m,
            _ => return,
        };

        let matched_chars = reversed[..mat.end()].chars().count();

        self.cursor.x = self.cursor.x.saturating_sub(matched_chars as u16);
    }

    fn move_to_next_line(&mut self) {
        self.cursor = Position {
            x: 0,
            y: self.cursor.y + 1,
        }
    }

    fn move_to_previous_line(&mut self) {
        self.cursor = Position {
            x: self.line_from_cursor(-1).chars().count() as u16,
            y: self.cursor.y - 1,
        }
    }
}
