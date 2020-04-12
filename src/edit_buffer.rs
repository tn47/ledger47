use crossterm::event::{KeyCode, KeyModifiers};
use log::trace;
use ropey::Rope;
use unicode_width::UnicodeWidthChar;

use std::{cmp, io};

use crate::event::Event;
use ledger::{
    core::{Error, Result},
    err_at,
};

const NEW_LINE_CHAR: char = '\n';

#[derive(Clone)]
struct Config {
    tabstop: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config { tabstop: 4 }
    }
}

pub struct EditRes {
    pub col_at: usize,
    pub row_at: usize,
    pub evnt: Option<Event>,
}

impl EditRes {
    #[inline]
    fn new(col_at: usize, row_at: usize, evnt: Option<Event>) -> EditRes {
        EditRes {
            col_at,
            row_at,
            evnt,
        }
    }
}

enum EditEvent {
    Noop,
    Esc,
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8, KeyModifiers),
    Char(char, KeyModifiers),
}

impl From<Event> for EditEvent {
    fn from(evnt: Event) -> EditEvent {
        let m = evnt.to_modifiers();
        let ctrl = m.contains(KeyModifiers::CONTROL);
        // let shift = m.contains(KeyModifiers::SHIFT);
        match evnt {
            Event::Key { code, modifiers } => match code {
                KeyCode::Backspace if m.is_empty() => EditEvent::Backspace,
                KeyCode::Enter if m.is_empty() => EditEvent::Enter,
                KeyCode::Left if m.is_empty() => EditEvent::Left,
                KeyCode::Right if m.is_empty() => EditEvent::Right,
                KeyCode::Up if m.is_empty() => EditEvent::Up,
                KeyCode::Down if m.is_empty() => EditEvent::Down,
                KeyCode::Home if m.is_empty() => EditEvent::Home,
                KeyCode::End if m.is_empty() => EditEvent::End,
                KeyCode::PageUp if m.is_empty() => EditEvent::PageUp,
                KeyCode::PageDown if m.is_empty() => EditEvent::PageDown,
                KeyCode::Tab if m.is_empty() => EditEvent::Tab,
                KeyCode::BackTab if m.is_empty() => EditEvent::BackTab,
                KeyCode::Delete if m.is_empty() => EditEvent::Delete,
                KeyCode::F(f) if m.is_empty() => EditEvent::F(f, modifiers),
                KeyCode::Char('[') if ctrl => EditEvent::Esc,
                KeyCode::Char(ch) if m.is_empty() => EditEvent::Char(ch, modifiers),
                KeyCode::Esc if m.is_empty() => EditEvent::Esc,
                KeyCode::Insert | KeyCode::Null => EditEvent::Noop,
                _ => EditEvent::Noop,
            },
            _ => EditEvent::Noop,
        }
    }
}

impl EditEvent {
    fn to_modifiers(evnt: &EditEvent) -> KeyModifiers {
        match evnt {
            EditEvent::F(_, modifiers) => modifiers.clone(),
            EditEvent::Char(_, modifiers) => modifiers.clone(),
            _ => KeyModifiers::empty(),
        }
    }
}

// all bits and pieces of content in a layer/page is managed by buffer.
// cursor is char_idx into buffer.
#[derive(Clone)]
pub struct Buffer {
    buf: Rope,
    cursor: usize,
    config: Config,
}

impl AsRef<Rope> for Buffer {
    fn as_ref(&self) -> &Rope {
        &self.buf
    }
}

impl Default for Buffer {
    fn default() -> Buffer {
        let bytes: Vec<u8> = vec![];
        Buffer {
            buf: Rope::from_reader(bytes.as_slice()).unwrap(),
            cursor: Default::default(),
            config: Default::default(),
        }
    }
}

impl Buffer {
    pub fn from_reader<R>(data: R) -> Result<Buffer>
    where
        R: io::Read,
    {
        let buf = err_at!(IOError, Rope::from_reader(data))?;
        Ok(Buffer {
            buf,
            cursor: 0,
            config: Default::default(),
        })
    }

    pub fn empty() -> Result<Buffer> {
        let bytes: Vec<u8> = vec![];
        let buf = err_at!(IOError, Rope::from_reader(bytes.as_slice()))?;
        Ok(Buffer {
            buf,
            cursor: 0,
            config: Default::default(),
        })
    }

    pub fn to_string(&self) -> String {
        self.as_ref().to_string()
    }

    pub fn view_lines(&self, from: usize) -> Vec<String> {
        self.as_ref()
            .lines_at(from)
            .map(|s| s.to_string().replace('\t', "    "))
            .collect()
    }

    pub fn handle_event(&mut self, evnt: Event) -> Result<EditRes> {
        use EditEvent::{BackTab, Backspace, Char, Delete, Down, End, Enter};
        use EditEvent::{Esc, Home, Insert, Left, Noop, PageDown, PageUp};
        use EditEvent::{Right, Tab, Up, F};

        let eevnt: EditEvent = evnt.clone().into();
        let cursr = self.cursor;

        let line_idx = self.buf.char_to_line(cursr);
        let start_idx = self.buf.line_to_char(line_idx);

        let ((col_at, row_at), evnt) = match eevnt {
            Char(ch, _) => {
                self.buf.insert_char(cursr, ch);
                (self.update_cursor(cursr + 1), None)
            }
            Backspace if cursr == 0 => ((0, 0), None),
            Backspace => {
                let new_cursor = cursr - 1;
                self.buf.remove(new_cursor..cursr);
                (self.update_cursor(new_cursor), None)
            }
            Enter => {
                self.buf.insert_char(cursr, NEW_LINE_CHAR);
                (self.update_cursor(cursr + 1), None)
            }
            Left if start_idx == cursr => (self.update_cursor(cursr), None),
            Left => (self.update_cursor(cursr - 1), None),
            Right => {
                if line_last_char(&self.buf, cursr) == cursr {
                    (self.update_cursor(cursr), None)
                } else {
                    (self.update_cursor(cursr + 1), None)
                }
            }
            Up if cursr == 0 => (self.update_cursor(cursr), None),
            Up => {
                let mut lines = self.buf.lines_at(line_idx);
                let (prev_line, cur_line) = (lines.prev(), lines.next());
                match (prev_line, cur_line) {
                    (None, _) => (self.update_cursor(cursr), None),
                    (Some(pline), Some(_)) => {
                        let row_at = line_idx - 1;
                        let col_at = cmp::min(pline.len_chars() - 1, cursr - start_idx);
                        trace!("pline {} {} {}", pline.to_string(), row_at, col_at);
                        (
                            self.update_cursor(self.buf.line_to_char(row_at) + col_at),
                            None,
                        )
                    }
                    _ => err_at!(Fatal, msg: format!("unreachable"))?,
                }
            }
            Down => {
                let mut lines = self.buf.lines_at(line_idx);
                let (cur_line, next_line) = (lines.next(), lines.next());
                match (cur_line, next_line) {
                    (None, _) => (self.update_cursor(cursr), None),
                    (Some(_), None) => (self.update_cursor(cursr), None),
                    (Some(_), Some(nline)) => {
                        let row_at = line_idx + 1;
                        let col_at = if nline.len_chars() > 0 {
                            cmp::min(nline.len_chars() - 1, cursr - start_idx)
                        } else {
                            0
                        };
                        (
                            self.update_cursor(self.buf.line_to_char(row_at) + col_at),
                            None,
                        )
                    }
                }
            }
            Home => (self.update_cursor(start_idx), None),
            End => {
                let new_cursor = line_last_char(&self.buf, cursr);
                (self.update_cursor(new_cursor), None)
            }
            Tab => {
                self.buf.insert_char(cursr, '\t');
                (self.update_cursor(cursr + 1), None)
            }
            Delete => {
                if cursr < line_last_char(&self.buf, cursr) {
                    self.buf.remove(cursr..(cursr + 1));
                }
                (self.update_cursor(cursr), None)
            }
            F(_, _) | BackTab | Insert | PageUp | PageDown | Noop | Esc => {
                (self.update_cursor(cursr), Some(evnt))
            }
        };

        Ok(EditRes {
            col_at,
            row_at,
            evnt,
        })
    }
}

impl Buffer {
    fn update_cursor(&mut self, new_cursor: usize) -> (usize, usize) {
        let (col_at, row_at) = {
            let row_at = self.buf.char_to_line(new_cursor);
            let col_at = new_cursor - self.buf.line_to_char(row_at);
            match self.buf.lines_at(row_at).next() {
                Some(line) => {
                    let a_col_at: usize = line
                        .to_string()
                        .chars()
                        .take(col_at)
                        .map(|ch| match ch {
                            '\t' => 4,
                            ch => ch.width().unwrap(),
                        })
                        .sum();
                    (a_col_at, row_at)
                }
                None => (col_at, row_at),
            }
        };

        self.cursor = new_cursor;
        (col_at, row_at)
    }
}

fn line_last_char(buf: &Rope, cursor: usize) -> usize {
    let line_idx = buf.char_to_line(cursor);
    let start_idx = buf.line_to_char(line_idx);
    let line = buf.line(line_idx);
    let chars: Vec<char> = line.chars().collect();
    let mut iter = chars.iter().rev();
    let n = match (iter.next(), iter.next()) {
        (Some('\n'), Some('\r')) => 2,
        (Some('\r'), Some('\n')) => 2,
        (Some('\n'), _) => 1,
        _ => 0,
    };
    trace!("line_last_char {} {} {}", start_idx, chars.len(), n);
    start_idx + chars.len() - n
}

#[cfg(test)]
#[path = "edit_buffer_test.rs"]
mod edit_buffer_test;
