use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ropey::Rope;

use std::io;

use ledger::core::{Error, Result};

const NEW_LINE_CHAR: char = '\n';

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
    fn to_modifiers(evnt: &InsertEvent) -> KeyModifiers {
        match evnt {
            InsertEvent::F(_, modifiers) => modifiers.clone(),
            InsertEvent::Char(_, modifiers) => modifiers.clone(),
            _ => KeyModifiers::empty(),
        }
    }
}

impl InsertEvent {
    fn handle_event(&self, buf: &mut Rope, cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        use InsertEvent::{BackTab, Backspace, Char, Delete, Down, End, Enter};
        use InsertEvent::{Esc, Home, Insert, Left, Noop, PageDown, PageUp};
        use InsertEvent::{Right, Tab, Up, F};

        let res = match self {
            Backspace if *cursor == 0 => EditRes::new(None, 0, Some(evnt)),
            Backspace => {
                let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
                if start_idx == 0 && start_idx == *cursor {
                    EditRes::new(None, 0, None) // consume as noop
                } else if start_idx == 0 {
                    let new_cursor = cursor.saturating_sub(1);
                    buf.remove(new_cursor..*cursor);
                    *cursor = new_cursor;
                    EditRes::new(Some(*cursor - start_idx), 0, None)
                } else if start_idx == *cursor {
                    let first_idx = line_first_char(buf, start_idx - 1);
                    let last_idx = line_last_char(buf, start_idx - 1);
                    buf.remove((last_idx + 1)..*cursor);
                    *cursor = last_idx + 1;
                    EditRes::new(Some(last_idx - first_idx), -1, None)
                } else {
                    let first_idx = line_first_char(buf, *cursor);
                    let new_cursor = cursor.saturating_sub(1);
                    buf.remove(new_cursor..*cursor);
                    *cursor = new_cursor;
                    EditRes::new(Some(first_idx - *cursor), 0, None)
                }
            }
            Enter => {
                buf.insert_char(*cursor, NEW_LINE_CHAR);
                *cursor += 1;
                EditRes::new(Some(0), 1, None)
            }
            Left => {
                let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
                if start_idx < *cursor {
                    *cursor -= 1;
                }
                EditRes::new(Some(*cursor - start_idx), 0, None)
            }
            Right => {
                let last_idx = line_last_char(buf, *cursor);
                let start_idx = line_first_char(buf, *cursor);
                if last_idx > *cursor {
                    *cursor += 1;
                }
                EditRes::new(Some(*cursor - start_idx), 0, None)
            }
            Up => {
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
                        EditRes::new(Some(col_at), -1, None)
                    }
                    None => EditRes::new(None, 0, None), // consume as noop
                }
            }
            Down => {
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
                        EditRes::new(Some(col_at), 1, None)
                    }
                    None => EditRes::new(None, 0, None), // consume as noop
                }
            }
            Home => {
                *cursor = line_first_char(buf, *cursor);
                EditRes::new(Some(0), 0, None)
            }
            End => {
                let first_idx = line_first_char(buf, *cursor);
                *cursor = line_last_char(buf, *cursor);
                EditRes::new(Some(*cursor - first_idx), 0, None)
            }
            Tab => {
                let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
                let cur_col = *cursor - start_idx;

                buf.insert_char(*cursor, '\t');
                *cursor += 1;
                EditRes::new(Some(cur_col + 1), 1, None)
            }
            Delete => {
                buf.remove(*cursor..(*cursor + 1));
                EditRes::new(None, 0, None)
            }
            Char(ch, _) => {
                let start_idx = buf.line_to_char(buf.char_to_line(*cursor));
                let cur_col = *cursor - start_idx;
                buf.insert_char(*cursor, *ch);
                *cursor += 1;
                EditRes::new(Some(cur_col + 1), 0, None)
            }
            F(_, _) | BackTab | Insert | PageUp | PageDown | Noop | Esc => {
                EditRes::new(None, 0, Some(evnt))
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
    fn to_modifiers(evnt: &NormalEvent) -> KeyModifiers {
        match evnt {
            NormalEvent::F(_, modifiers) => modifiers.clone(),
            NormalEvent::Char(_, modifiers) => modifiers.clone(),
            _ => KeyModifiers::empty(),
        }
    }
}

impl NormalEvent {
    fn handle_event(&self, _buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
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
    fn to_modifiers(evnt: &ReplaceEvent) -> KeyModifiers {
        match evnt {
            ReplaceEvent::F(_, modifiers) => modifiers.clone(),
            ReplaceEvent::Char(_, modifiers) => modifiers.clone(),
            _ => KeyModifiers::empty(),
        }
    }
}

impl ReplaceEvent {
    fn handle_event(&self, _buf: &mut Rope, _cursor: &mut usize, evnt: Event) -> Result<EditRes> {
        Ok(EditRes::new(None, 0, Some(evnt)))
    }
}

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
