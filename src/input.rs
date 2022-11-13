use crossterm::event::{KeyCode, KeyEvent};

// TODO change capacity
const INPUT_BUFFER_CAPACITY: usize = 10;
const CURSOR_BEGINNING: CursorPosition = CursorPosition::Pos(0);

/// Handler for inputs trough all [`InputMode`] modes.
#[derive(Debug)]
pub struct InputHandler {
    /// Input content.
    buffer: String,

    /// Used because of chars made from multiple bytes.
    char_count: usize,

    /// Keeps count of multi byte chars.
    multi_byte_chars: usize,

    // TODO maybe use a single number instead
    /// Keeps track of the terminal cursor position.
    cursor: CursorPosition,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(INPUT_BUFFER_CAPACITY),
            char_count: 0,
            multi_byte_chars: 0,
            cursor: CursorPosition::End,
        }
    }

    pub fn process_input(&mut self, event: KeyEvent) -> Option<Interruption> {
        let key = event.code;
        match key {
            KeyCode::Backspace => self.backspace_key(),
            KeyCode::Left => self.left_key(),
            KeyCode::Right => self.right_key(),
            KeyCode::Up => self.cursor = CursorPosition::End,
            KeyCode::Down => self.cursor = CURSOR_BEGINNING,
            KeyCode::Home => self.cursor = CURSOR_BEGINNING,
            KeyCode::End => self.cursor = CursorPosition::End,
            KeyCode::Delete => self.delete_key(),
            KeyCode::Char(c) => self.add_char(c),
            KeyCode::Enter => return self.enter_key(),
            KeyCode::Esc => return self.esc_key(),
            _ => (),
        };
        None
    }

    fn add_char(&mut self, c: char) {
        self.char_count += 1;
        if c.len_utf8() != 1 {
            self.multi_byte_chars += 1;
        }

        match &mut self.cursor {
            CursorPosition::Pos(index) => {
                if self.multi_byte_chars == 0 {
                    self.buffer.insert(*index, c);
                } else {
                    let byte_pos = self.buffer.char_indices().nth(*index).unwrap().0;
                    self.buffer.insert(byte_pos, c);
                }
                // Increment because cursor is being moved to the right after char input
                *index += 1;
            }
            CursorPosition::End => self.buffer.push(c),
        }
    }

    fn left_key(&mut self) {
        match self.cursor {
            CursorPosition::Pos(index) => {
                if index > 0 {
                    self.cursor = CursorPosition::Pos(index - 1)
                }
            }
            CursorPosition::End => {
                if self.char_count != 0 {
                    self.cursor = CursorPosition::Pos(self.char_count - 1)
                }
            }
        }
    }

    fn right_key(&mut self) {
        match self.cursor {
            CursorPosition::Pos(index) => {
                let new_pos = index + 1;
                self.cursor = if new_pos == self.char_count {
                    CursorPosition::End
                } else {
                    CursorPosition::Pos(new_pos)
                }
            }
            // Do nothing since it's already at the end
            CursorPosition::End => (),
        }
    }

    fn delete_key(&mut self) {
        match self.cursor {
            CursorPosition::Pos(index) => {
                self.remove_char(index);

                if index == self.char_count {
                    self.cursor = CursorPosition::End
                }
            }
            // Do nothing since there is no chars after the ending
            CursorPosition::End => (),
        };
    }

    fn backspace_key(&mut self) {
        match self.cursor {
            CursorPosition::Pos(index) => {
                if index > 0 {
                    // Backspace removes the char left from the cursor
                    self.remove_char(index - 1);
                    if self.char_count == 0 {
                        self.cursor = CursorPosition::End
                    } else {
                        self.left_key()
                    }
                }
            }
            CursorPosition::End => {
                if !self.buffer.is_empty() {
                    self.remove_char(self.char_count - 1);
                }
            }
        }
    }

    fn enter_key(&mut self) -> Option<Interruption> {
        let buffer = self.buffer.drain(..).collect();
        self.clear();
        Some(Interruption::Enter(buffer))
    }

    fn esc_key(&mut self) -> Option<Interruption> {
        if self.char_count == 0 {
            // Exit the editing mode if there is nothing in the buffer.
            assert!(self.buffer.is_empty());
            assert_eq!(self.cursor, CursorPosition::End);
            Some(Interruption::Esc)
        } else {
            self.clear();
            None
        }
    }

    fn remove_char(&mut self, index: usize) {
        assert!(index < self.char_count);

        self.char_count -= 1;
        if self.multi_byte_chars == 0 {
            self.buffer.remove(index);
        } else {
            let byte_pos = self.buffer.char_indices().nth(index).unwrap().0;
            if self.buffer.remove(byte_pos).len_utf8() != 1 {
                self.multi_byte_chars -= 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.char_count = 0;
        self.multi_byte_chars = 0;
        self.cursor = CursorPosition::End;
    }

    pub fn current_input(&self) -> &str {
        &self.buffer
    }

    pub fn get_cursor_position(&self) -> usize {
        match self.cursor {
            CursorPosition::Pos(i) => i,
            CursorPosition::End => self.char_count,
        }
    }
}

/// Cursor has a unique End position because it is going to be at the
/// end of the input buffer most of the time.
#[derive(Debug, PartialEq)]
enum CursorPosition {
    Pos(usize),
    End,
}

/// Events which occur when the user tries exiting the Editing mode.
#[derive(Debug)]
pub enum Interruption {
    /// Exit the Editing mode and return the input.
    Enter(String),

    /// Exit the Editing mode.
    Esc,
}

impl std::fmt::Display for InputHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "buf: {}, len: {}, chars: {}, cursor pos: {}",
            self.buffer,
            self.buffer.len(),
            self.char_count,
            self.get_cursor_position()
        )
    }
}
