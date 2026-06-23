use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terminal dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// Terminal character attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharAttributes {
    pub fg_color: Option<u8>,
    pub bg_color: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

impl Default for CharAttributes {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
            reverse: false,
        }
    }
}

/// Terminal character with attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalChar {
    pub ch: char,
    pub attrs: CharAttributes,
}

impl TerminalChar {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            attrs: CharAttributes::default(),
        }
    }

    pub fn with_attrs(ch: char, attrs: CharAttributes) -> Self {
        Self { ch, attrs }
    }
}

impl Default for TerminalChar {
    fn default() -> Self {
        Self::new(' ')
    }
}

/// Cursor position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

impl CursorPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self::new(0, 0)
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self::origin()
    }
}

/// Complete terminal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub size: TerminalSize,
    pub cursor: CursorPosition,
    pub buffer: Vec<Vec<TerminalChar>>,
    pub title: String,
    pub cursor_visible: bool,
}

impl TerminalState {
    pub fn new(size: TerminalSize) -> Self {
        let buffer = vec![
            vec![TerminalChar::default(); size.width as usize]; 
            size.height as usize
        ];

        Self {
            size,
            cursor: CursorPosition::origin(),
            buffer,
            title: String::new(),
            cursor_visible: true,
        }
    }

    /// Get character at position
    pub fn get_char(&self, x: u16, y: u16) -> Option<&TerminalChar> {
        if x < self.size.width && y < self.size.height {
            self.buffer
                .get(y as usize)
                .and_then(|row| row.get(x as usize))
        } else {
            None
        }
    }

    /// Set character at position
    pub fn set_char(&mut self, x: u16, y: u16, ch: TerminalChar) {
        if x < self.size.width && y < self.size.height {
            if let Some(row) = self.buffer.get_mut(y as usize) {
                if let Some(cell) = row.get_mut(x as usize) {
                    *cell = ch;
                }
            }
        }
    }

    /// Get text content as string
    pub fn get_text(&self) -> String {
        self.buffer
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.ch)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get text content of a specific line
    pub fn get_line_text(&self, y: u16) -> Option<String> {
        if y < self.size.height {
            Some(
                self.buffer[y as usize]
                    .iter()
                    .map(|cell| cell.ch)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            )
        } else {
            None
        }
    }

    /// Check if text exists in terminal
    pub fn contains_text(&self, text: &str) -> bool {
        self.get_text().contains(text)
    }

    /// Find text position in terminal
    pub fn find_text(&self, text: &str) -> Option<CursorPosition> {
        let content = self.get_text();
        if let Some(pos) = content.find(text) {
            // Convert byte position to line/column
            let lines: Vec<&str> = content.lines().collect();
            let mut char_count = 0;
            
            for (line_idx, line) in lines.iter().enumerate() {
                if char_count + line.len() >= pos {
                    let col = pos - char_count;
                    return Some(CursorPosition::new(col as u16, line_idx as u16));
                }
                char_count += line.len() + 1; // +1 for newline
            }
        }
        None
    }

    /// Resize terminal
    pub fn resize(&mut self, new_size: TerminalSize) {
        if new_size == self.size {
            return;
        }

        let mut new_buffer = vec![
            vec![TerminalChar::default(); new_size.width as usize]; 
            new_size.height as usize
        ];

        // Copy existing content
        let copy_height = std::cmp::min(self.size.height, new_size.height);
        let copy_width = std::cmp::min(self.size.width, new_size.width);

        for y in 0..copy_height {
            for x in 0..copy_width {
                if let Some(ch) = self.get_char(x, y) {
                    new_buffer[y as usize][x as usize] = ch.clone();
                }
            }
        }

        self.buffer = new_buffer;
        self.size = new_size;

        // Adjust cursor position if necessary
        if self.cursor.x >= new_size.width {
            self.cursor.x = new_size.width - 1;
        }
        if self.cursor.y >= new_size.height {
            self.cursor.y = new_size.height - 1;
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new(TerminalSize::default())
    }
}