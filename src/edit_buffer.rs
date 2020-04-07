use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use log::trace;
use ropey::Rope;

use std::{cmp, io};

use ledger::core::{Error, Result};

const NEW_LINE_CHAR: char = '\n';

macro_rules! update_cursor {
    ($buf:expr, $cursor:expr, $new_cursor:expr, $evnt:expr) => {{
        let (col_at, row_at) = {
            let row_at = $buf.char_to_line($new_cursor);
            let col_at = $new_cursor - $buf.line_to_char(row_at);
            (col_at, row_at)
        };

        $cursor = $new_cursor;
        EditRes::new(col_at, row_at, $evnt)
    }};
}

macro_rules! common_impl_event {
    ($evty:ident) => {
        impl $evty {
            fn to_modifiers(evnt: &$evty) -> KeyModifiers {
                match evnt {
                    $evty::F(_, modifiers) => modifiers.clone(),
                    $evty::Char(_, modifiers) => modifiers.clone(),
                    _ => KeyModifiers::empty(),
                }
            }
        }
    };
}

common_impl_event!(InsertEvent);
common_impl_event!(NormalEvent);
common_impl_event!(ReplaceEvent);

enum InsertEvent {
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

impl From<Event> for InsertEvent {
    fn from(evnt: Event) -> InsertEvent {
        match evnt {
            Event::Key(KeyEvent { code, modifiers }) => match code {
                KeyCode::Backspace if to_modifiers(&evnt).is_empty() => InsertEvent::Backspace,
                KeyCode::Enter if to_modifiers(&evnt).is_empty() => InsertEvent::Enter,
                KeyCode::Left if to_modifiers(&evnt).is_empty() => InsertEvent::Left,
                KeyCode::Right if to_modifiers(&evnt).is_empty() => InsertEvent::Right,
                KeyCode::Up if to_modifiers(&evnt).is_empty() => InsertEvent::Up,
                KeyCode::Down if to_modifiers(&evnt).is_empty() => InsertEvent::Down,
                KeyCode::Home if to_modifiers(&evnt).is_empty() => InsertEvent::Home,
                KeyCode::End if to_modifiers(&evnt).is_empty() => InsertEvent::End,
                KeyCode::PageUp if to_modifiers(&evnt).is_empty() => InsertEvent::PageUp,
                KeyCode::PageDown if to_modifiers(&evnt).is_empty() => InsertEvent::PageDown,
                KeyCode::Tab if to_modifiers(&evnt).is_empty() => InsertEvent::Tab,
                KeyCode::BackTab if to_modifiers(&evnt).is_empty() => InsertEvent::BackTab,
                KeyCode::Delete if to_modifiers(&evnt).is_empty() => InsertEvent::Delete,
                KeyCode::F(f) if to_modifiers(&evnt).is_empty() => InsertEvent::F(f, modifiers),
                KeyCode::Char(ch) if to_modifiers(&evnt).is_empty() => {
                    InsertEvent::Char(ch, modifiers)
                }
                KeyCode::Esc if to_modifiers(&evnt).is_empty() => InsertEvent::Esc,
                KeyCode::Insert | KeyCode::Null => InsertEvent::Noop,
                _ => InsertEvent::Noop,
            },
            Event::Mouse(_) => InsertEvent::Noop,
            Event::Resize(_, _) => InsertEvent::Noop,
        }
    }
}

impl InsertEvent {
    fn handle_event(&self, buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        use InsertEvent::{BackTab, Backspace, Char, Delete, Down, End, Enter};
        use InsertEvent::{Esc, Home, Insert, Left, Noop, PageDown, PageUp};
        use InsertEvent::{Right, Tab, Up, F};

        let line_idx = buf.char_to_line(*cursor);
        let start_idx = buf.line_to_char(line_idx);

        let res = match self {
            Char(ch, _) => {
                buf.insert_char(*cursor, *ch);
                update_cursor!(buf, *cursor, *cursor + 1, None)
            }
            Backspace if *cursor == 0 => EditRes::new(0, 0, Some(evnt)),
            Backspace => {
                let new_cursor = *cursor - 1;
                buf.remove(new_cursor..*cursor);
                update_cursor!(buf, *cursor, new_cursor, None)
            }
            Enter => {
                buf.insert_char(*cursor, NEW_LINE_CHAR);
                update_cursor!(buf, *cursor, *cursor + 1, None)
            }
            Left if start_idx == *cursor => update_cursor!(buf, *cursor, *cursor, None),
            Left => update_cursor!(buf, *cursor, *cursor - 1, None),
            Right => {
                if line_last_char(buf, *cursor) == *cursor {
                    update_cursor!(buf, *cursor, *cursor, None)
                } else {
                    update_cursor!(buf, *cursor, *cursor + 1, None)
                }
            }
            Up if *cursor == 0 => update_cursor!(buf, *cursor, *cursor, Some(evnt)),
            Up => {
                let mut lines = buf.lines_at(line_idx);
                let (prev_line, cur_line) = (lines.prev(), lines.next());
                match (prev_line, cur_line) {
                    (None, _) => update_cursor!(buf, *cursor, *cursor, None),
                    (Some(pline), Some(_)) => {
                        let row_at = line_idx - 1;
                        let col_at = cmp::min(pline.len_chars(), *cursor - start_idx);
                        update_cursor!(buf, *cursor, buf.line_to_char(row_at) + col_at, None)
                    }
                    _ => err_at!(Fatal, msg: format!("unreachable"))?,
                }
            }
            Down => {
                let mut lines = buf.lines_at(line_idx);
                let (cur_line, next_line) = (lines.next(), lines.next());
                match (cur_line, next_line) {
                    (None, _) => update_cursor!(buf, *cursor, *cursor, None),
                    (Some(_), None) => update_cursor!(buf, *cursor, *cursor, None),
                    (Some(_), Some(nline)) => {
                        let row_at = line_idx + 1;
                        let col_at = cmp::min(nline.len_chars(), *cursor - start_idx);
                        update_cursor!(buf, *cursor, buf.line_to_char(row_at) + col_at, None)
                    }
                }
            }
            Home => update_cursor!(buf, *cursor, start_idx, None),
            End => {
                let new_cursor = line_last_char(buf, *cursor);
                update_cursor!(buf, *cursor, new_cursor, None)
            }
            Tab => {
                buf.insert_char(*cursor, '\t');
                update_cursor!(buf, *cursor, *cursor + 1, None)
            }
            Delete => {
                if *cursor < line_last_char(buf, *cursor) {
                    buf.remove(*cursor..(*cursor + 1));
                }
                update_cursor!(buf, *cursor, *cursor, None)
            }
            F(_, _) | BackTab | Insert | PageUp | PageDown | Noop | Esc => {
                update_cursor!(buf, *cursor, *cursor, Some(evnt))
            }
        };

        Ok(res)
    }
}

enum NormalEvent {
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

impl From<Event> for NormalEvent {
    fn from(evnt: Event) -> NormalEvent {
        match evnt {
            Event::Key(KeyEvent { code, modifiers }) => match code {
                KeyCode::Backspace => NormalEvent::Backspace,
                KeyCode::Enter => NormalEvent::Enter,
                KeyCode::Left => NormalEvent::Left,
                KeyCode::Right => NormalEvent::Right,
                KeyCode::Up => NormalEvent::Up,
                KeyCode::Down => NormalEvent::Down,
                KeyCode::Home => NormalEvent::Home,
                KeyCode::End => NormalEvent::End,
                KeyCode::PageUp => NormalEvent::PageUp,
                KeyCode::PageDown => NormalEvent::PageDown,
                KeyCode::Tab => NormalEvent::Tab,
                KeyCode::BackTab => NormalEvent::BackTab,
                KeyCode::Delete => NormalEvent::Delete,
                KeyCode::F(f) => NormalEvent::F(f, modifiers),
                KeyCode::Char(ch) => NormalEvent::Char(ch, modifiers),
                KeyCode::Insert | KeyCode::Null => NormalEvent::Noop,
                KeyCode::Esc => NormalEvent::Esc,
            },
            Event::Mouse(_) => NormalEvent::Noop,
            Event::Resize(_, _) => NormalEvent::Noop,
        }
    }
}

impl NormalEvent {
    fn handle_event(&self, buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(update_cursor!(buf, *cursor, *cursor, Some(evnt)))
    }
}

enum ReplaceEvent {
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

impl From<Event> for ReplaceEvent {
    fn from(evnt: Event) -> ReplaceEvent {
        match evnt {
            Event::Key(KeyEvent { code, modifiers }) => match code {
                KeyCode::Backspace => ReplaceEvent::Backspace,
                KeyCode::Enter => ReplaceEvent::Enter,
                KeyCode::Left => ReplaceEvent::Left,
                KeyCode::Right => ReplaceEvent::Right,
                KeyCode::Up => ReplaceEvent::Up,
                KeyCode::Down => ReplaceEvent::Down,
                KeyCode::Home => ReplaceEvent::Home,
                KeyCode::End => ReplaceEvent::End,
                KeyCode::PageUp => ReplaceEvent::PageUp,
                KeyCode::PageDown => ReplaceEvent::PageDown,
                KeyCode::Tab => ReplaceEvent::Tab,
                KeyCode::BackTab => ReplaceEvent::BackTab,
                KeyCode::Delete => ReplaceEvent::Delete,
                KeyCode::F(f) => ReplaceEvent::F(f, modifiers),
                KeyCode::Char(ch) => ReplaceEvent::Char(ch, modifiers),
                KeyCode::Insert | KeyCode::Null => ReplaceEvent::Noop,
                KeyCode::Esc => ReplaceEvent::Esc,
            },
            Event::Mouse(_) => ReplaceEvent::Noop,
            Event::Resize(_, _) => ReplaceEvent::Noop,
        }
    }
}

impl ReplaceEvent {
    fn handle_event(&self, buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(update_cursor!(buf, *cursor, *cursor, Some(evnt)))
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

// all bits and pieces of content in a layer/page is managed by buffer.
#[derive(Clone)]
pub enum Buffer {
    Normal { buf: Rope, cursor: usize }, // cursor is char_idx into buffer.
    Insert { buf: Rope, cursor: usize }, // cursor is char_idx into buffer.
    Replace { buf: Rope, cursor: usize }, // cursor is char_idx into buffer.
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

    pub fn to_lines(&self, from: usize, till: usize) -> Vec<String> {
        let buf = match self {
            Buffer::Normal { buf, .. } => buf,
            Buffer::Insert { buf, .. } => buf,
            Buffer::Replace { buf, .. } => buf,
        };
        (from..till)
            .map(|line_idx| buf.line(line_idx).to_string())
            .collect()
    }
}

impl Buffer {
    pub fn handle_event(&mut self, evnt: Event) -> Result<EditRes> {
        match self {
            Buffer::Normal { buf, cursor } => {
                let ne: NormalEvent = evnt.clone().into();
                ne.handle_event(buf, cursor, evnt)
            }
            Buffer::Insert { buf, cursor } => {
                let ie: InsertEvent = evnt.clone().into();
                ie.handle_event(buf, cursor, evnt)
            }
            Buffer::Replace { buf, cursor } => {
                let re: ReplaceEvent = evnt.clone().into();
                re.handle_event(buf, cursor, evnt)
            }
        }
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

fn to_modifiers(evnt: &Event) -> KeyModifiers {
    match evnt {
        Event::Resize(_, _) => KeyModifiers::empty(),
        Event::Key(KeyEvent { modifiers, .. }) => modifiers.clone(),
        Event::Mouse(MouseEvent::Up(_, _, _, modifiers)) => modifiers.clone(),
        Event::Mouse(MouseEvent::Down(_, _, _, modifiers)) => modifiers.clone(),
        Event::Mouse(MouseEvent::Drag(_, _, _, modifiers)) => modifiers.clone(),
        Event::Mouse(MouseEvent::ScrollDown(_, _, modifiers)) => modifiers.clone(),
        Event::Mouse(MouseEvent::ScrollUp(_, _, modifiers)) => modifiers.clone(),
    }
}

#[cfg(test)]
#[path = "edit_buffer_test.rs"]
mod edit_buffer_test;
