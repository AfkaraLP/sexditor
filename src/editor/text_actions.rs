use crate::editor::{Editor, Position};

pub trait TextAction {
    fn insert_char(&mut self, pos: &Position, c: char);
    fn remove_char(&mut self, pos: &Position);
    fn get_byte_offset(&self, pos: &Position) -> usize;
}

impl TextAction for Editor {
    fn insert_char(&mut self, pos: &Position, c: char) {
        self.file_text.insert(self.get_byte_offset(pos), c);
    }

    fn remove_char(&mut self, pos: &Position) {
        let byte_offset = self.get_byte_offset(pos);
        if byte_offset >= self.file_text.len() {
            self.file_text.pop();
            return;
        }
        self.file_text.remove(self.get_byte_offset(pos));
    }

    fn get_byte_offset(&self, pos: &Position) -> usize {
        let mut offset = 0usize;
        for (i, line) in self.file_text.lines().enumerate() {
            if i == pos.y as usize {
                let x = pos.x as usize;
                offset += line
                    .char_indices()
                    .nth(x)
                    .map(|(byte_idx, _)| byte_idx)
                    .unwrap_or(line.len());
                break;
            } else {
                offset += line.len() + 1;
            }
        }
        offset
    }
}
