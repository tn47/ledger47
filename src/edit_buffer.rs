use crossterm::event::KeyCode;
use ropey::Rope;

use std::io;

use crate::event::Event;
use ledger::core::{Error, Result};

const NEW_LINE_CHAR: char = '\n';

pub struct EditRes {
    pub col_at: Option<usize>,
    pub row_by: isize,
    pub evnt: Option<Event>,
}

impl EditRes {
    #[inline]
    fn new(col_at: Option<usize>, row_by: isize, evnt: Option<Event>) -> EditRes {
        EditRes {
            col_at,
            row_by,
            evnt,
        }
    }
}

// all bits and pieces of content in a layer/page is managed by buffer.
#[derive(Clone)]
pub enum Buffer {
    Normal { buf: Rope, cursor: usize },
    Insert { buf: Rope, cursor: usize },
    Replace { buf: Rope, cursor: usize },
}

impl Default for Buffer {
    fn default() -> Buffer {
        let bytes: Vec<u8> = vec![];
        Buffer::Normal {
            buf: Rope::from_reader(bytes.as_slice()).unwrap(),
            cursor: Default::default(),
        }
    }
}

impl Buffer {
    pub fn from_reader<R>(data: R) -> Result<Buffer>
    where
        R: io::Read,
    {
        let buf = err_at!(IOError, Rope::from_reader(data))?;
        Ok(Buffer::Normal { buf, cursor: 0 })
    }

    pub fn empty() -> Result<Buffer> {
        let bytes: Vec<u8> = vec![];
        let buf = err_at!(IOError, Rope::from_reader(bytes.as_slice()))?;
        Ok(Buffer::Normal { buf, cursor: 0 })
    }

    pub fn change_to_insert(self) -> Self {
        use Buffer::{Insert, Normal, Replace};
        match self {
            Normal { buf, cursor } => Insert { buf, cursor },
            Insert { buf, cursor } => Insert { buf, cursor },
            Replace { buf, cursor } => Insert { buf, cursor },
        }
    }

    pub fn change_to_replace(self) -> Self {
        use Buffer::{Insert, Normal, Replace};
        match self {
            Normal { buf, cursor } => Replace { buf, cursor },
            Insert { buf, cursor } => Replace { buf, cursor },
            Replace { buf, cursor } => Replace { buf, cursor },
        }
    }

    pub fn change_to_normal(self) -> Self {
        use Buffer::{Insert, Normal, Replace};
        match self {
            Normal { buf, cursor } => Normal { buf, cursor },
            Insert { buf, cursor } => Normal { buf, cursor },
            Replace { buf, cursor } => Normal { buf, cursor },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Buffer::Normal { buf, .. } => buf,
            Buffer::Insert { buf, .. } => buf,
            Buffer::Replace { buf, .. } => buf,
        }
        .to_string()
    }

    pub fn to_lines(&self) -> Vec<String> {
        match self {
            Buffer::Normal { buf, .. } => {
                buf.lines().map(|l| l.to_string()).collect::<Vec<String>>()
            }
            Buffer::Insert { buf, .. } => {
                buf.lines().map(|l| l.to_string()).collect::<Vec<String>>()
            }
            Buffer::Replace { buf, .. } => {
                buf.lines().map(|l| l.to_string()).collect::<Vec<String>>()
            }
        }
    }
}

impl Buffer {
    // TODO: Handle backspace for new-line chars other than '\n'.
    fn do_key_backspace(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if *cursor == 0 || !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            if start_idx == 0 && start_idx == *cursor {
                Ok(EditRes::new(None, 0, None)) // consume as noop
            } else if start_idx == 0 {
                let new_cursor = cursor.saturating_sub(1);
                buf.remove(new_cursor..*cursor);
                *cursor = new_cursor;
                Ok(EditRes::new(Some(*cursor - start_idx), 0, None))
            } else if start_idx == *cursor {
                let first_idx = line_first_char(buf, start_idx - 1);
                let last_idx = line_last_char(buf, start_idx - 1);
                buf.remove((last_idx + 1)..*cursor);
                *cursor = last_idx + 1;
                Ok(EditRes::new(Some(last_idx - first_idx), -1, None))
            } else {
                let first_idx = line_first_char(buf, *cursor);
                let new_cursor = cursor.saturating_sub(1);
                buf.remove(new_cursor..*cursor);
                *cursor = new_cursor;
                Ok(EditRes::new(Some(first_idx - *cursor), 0, None))
            }
        }
    }

    fn do_key_enter(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            buf.insert_char(*cursor, NEW_LINE_CHAR);
            *cursor += 1;
            Ok(EditRes::new(Some(0), 1, None))
        }
    }

    fn do_key_left(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            if start_idx < *cursor {
                *cursor -= 1;
            }
            Ok(EditRes::new(Some(*cursor - start_idx), 0, None))
        }
    }

    fn do_key_right(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            let last_idx = line_last_char(buf, *cursor);
            let start_idx = line_first_char(buf, *cursor);
            if last_idx > *cursor {
                *cursor += 1;
            }
            Ok(EditRes::new(Some(*cursor - start_idx), 0, None))
        }
    }

    fn do_key_up(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt)))
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            let cur_col = *cursor - start_idx;
            let mut lines = buf.lines_at(buf.char_to_line(*cursor));
            match lines.prev() {
                Some(_) => {
                    let prev_start_idx = buf.line_to_char(buf.char_to_line(*cursor) - 1);
                    let prev_last_idx = line_last_char(buf, start_idx);
                    let col_at = if (prev_last_idx - prev_start_idx) < cur_col {
                        *cursor = prev_last_idx;
                        prev_last_idx - prev_start_idx
                    } else {
                        *cursor = prev_start_idx + cur_col;
                        cur_col
                    };
                    Ok(EditRes::new(Some(col_at), -1, None))
                }
                None => Ok(EditRes::new(None, 0, None)), // consume as noop
            }
        }
    }

    fn do_key_down(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt)))
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            let cur_col = *cursor - start_idx;
            let mut lines = buf.lines_at(buf.char_to_line(*cursor));
            match lines.next() {
                Some(_) => {
                    let next_start_idx = buf.line_to_char(buf.char_to_line(*cursor) + 1);
                    let next_last_idx = line_last_char(buf, start_idx);
                    let col_at = if (next_last_idx - next_start_idx) < cur_col {
                        *cursor = next_last_idx;
                        next_last_idx - next_start_idx
                    } else {
                        *cursor = next_start_idx + cur_col;
                        cur_col
                    };
                    Ok(EditRes::new(Some(col_at), 1, None))
                }
                None => Ok(EditRes::new(None, 0, None)), // consume as noop
            }
        }
    }

    fn do_key_home(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt)))
        } else {
            *cursor = line_first_char(buf, *cursor);
            Ok(EditRes::new(Some(0), 0, None))
        }
    }

    fn do_key_end(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt)))
        } else {
            let first_idx = line_first_char(buf, *cursor);
            *cursor = line_last_char(buf, *cursor);
            Ok(EditRes::new(Some(*cursor - first_idx), 0, None))
        }
    }

    fn do_key_pageup(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_pagedown(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_tab(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt)))
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            let cur_col = *cursor - start_idx;

            buf.insert_char(*cursor, '\t');
            *cursor += 1;
            Ok(EditRes::new(Some(cur_col + 1), 1, None))
        }
    }

    fn do_key_backtab(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_delete(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            buf.remove(*cursor..(*cursor + 1));
            Ok(EditRes::new(None, 0, None))
        }
    }

    fn do_key_insert(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_fkey(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_char(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        if !evnt.to_modifier().is_empty() {
            Ok(EditRes::new(None, 0, Some(evnt))) // noop
        } else {
            let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
            let cur_col = *cursor - start_idx;
            match evnt {
                Event::Key {
                    code: KeyCode::Char(ch),
                    ..
                } => {
                    buf.insert_char(*cursor, '\t');
                    *cursor += 1;
                    Ok(EditRes::new(Some(cur_col + 1), 0, None))
                }
                _ => err_at!(Fatal, msg: format!("unreachable")),
            }
        }
    }

    fn do_key_null(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }

    fn do_key_esc(_buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }
}

impl Buffer {
    pub fn handle_event(&mut self, evnt: Event) -> Result<EditRes> {
        match self {
            Buffer::Normal { buf, cursor } => Self::handle_insert_event(buf, cursor, evnt),
            Buffer::Insert { buf, cursor } => Self::handle_normal_event(buf, cursor, evnt),
            Buffer::Replace { buf, cursor } => Self::handle_replace_event(buf, cursor, evnt),
        }
    }

    pub fn handle_insert_event(buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        match &evnt {
            Event::Key { code, .. } => match code {
                KeyCode::Backspace => Self::do_key_backspace(buf, cursor, evnt),
                KeyCode::Enter => Self::do_key_enter(buf, cursor, evnt),
                KeyCode::Left => Self::do_key_left(buf, cursor, evnt),
                KeyCode::Right => Self::do_key_right(buf, cursor, evnt),
                KeyCode::Up => Self::do_key_up(buf, cursor, evnt),
                KeyCode::Down => Self::do_key_down(buf, cursor, evnt),
                KeyCode::Home => Self::do_key_home(buf, cursor, evnt),
                KeyCode::End => Self::do_key_end(buf, cursor, evnt),
                KeyCode::PageUp => Self::do_key_pageup(buf, cursor, evnt),
                KeyCode::PageDown => Self::do_key_pagedown(buf, cursor, evnt),
                KeyCode::Tab => Self::do_key_tab(buf, cursor, evnt),
                KeyCode::BackTab => Self::do_key_backtab(buf, cursor, evnt),
                KeyCode::Delete => Self::do_key_delete(buf, cursor, evnt),
                KeyCode::Insert => Self::do_key_insert(buf, cursor, evnt),
                KeyCode::F(_) => Self::do_key_fkey(buf, cursor, evnt),
                KeyCode::Char(_) => Self::do_key_char(buf, cursor, evnt),
                KeyCode::Null => Self::do_key_null(buf, cursor, evnt),
                KeyCode::Esc => Self::do_key_esc(buf, cursor, evnt),
            },
            Event::Resize { .. }
            | Event::MouseDown { .. }
            | Event::MouseUp { .. }
            | Event::MouseDrag { .. }
            | Event::MouseScrollDown { .. }
            | Event::MouseScrollUp { .. } => Ok(EditRes::new(None, 0, Some(evnt))),
        }
    }

    pub fn handle_normal_event(
        _buf: &mut Rope,
        _cursor: &mut usize,
        _evnt: Event,
    ) -> Result<EditRes> {
        todo!()
    }

    pub fn handle_replace_event(
        _buf: &mut Rope,
        _cursor: &mut usize,
        _evnt: Event,
    ) -> Result<EditRes> {
        todo!()
    }
}

fn line_last_char(buf: &Rope, cursor: usize) -> usize {
    let start_idx = buf.line_to_char(buf.char_to_line(cursor));
    let line = buf.line(buf.char_to_line(cursor));
    let line_len = line.chars().len();
    let chars: Vec<char> = line.chars().collect();
    let mut iter = chars.iter().rev();
    let n = match (iter.next(), iter.next()) {
        (Some('\n'), Some('\r')) => line_len - 2 - 1,
        (Some('\r'), Some('\n')) => line_len - 2 - 1,
        (Some('\n'), _) => line_len - 1 - 1,
        (Some(_), _) => line_len - 1,
        (None, _) => line_len - 1,
    };
    start_idx + n
}

#[inline]
fn line_first_char(buf: &Rope, cursor: usize) -> usize {
    buf.line_to_char(buf.char_to_line(cursor))
}

#[cfg(test)]
#[path = "edit_buffer_test.rs"]
mod edit_buffer_test;
