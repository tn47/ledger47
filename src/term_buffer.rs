use crossterm::event::Event;
use ropey::Rope;

use std::io;

use ledger::core::{Error, Result};

// all bits and pieces of content in a layer/page is managed by buffer.
#[derive(Clone)]
pub enum Buffer {
    Normal {
        buf: Rope,
        window: Vec<u64>,
        cursor: (u64, u64),
    },
    Insert {
        buf: Rope,
        window: Vec<u64>,
        cursor: (u64, u64),
    },
    Replace {
        buf: Rope,
        window: Vec<u64>,
        cursor: (u64, u64),
    },
}

impl Default for Buffer {
    fn default() -> Buffer {
        let bytes: Vec<u8> = vec![];
        Buffer::Normal {
            buf: Rope::from_reader(bytes.as_slice()).unwrap(),
            window: Default::default(),
            cursor: Default::default(),
        }
    }
}

impl Buffer {
    //TODO
    //pub fn new_read_only<R>(data: R) -> Result<Buffer>
    //where
    //    R: io::Read,
    //{
    //    let buf = Rope::from_reader(data)?;
    //    let window: Vec<u64> = buf.lines().map(|rs| rs.chars().len() as u64).collect();
    //    Buffer::ReadOnly {
    //        buf,
    //        window,
    //        cursor: (1, 1),
    //    }
    //}

    pub fn new<R>(data: R) -> Result<Buffer>
    where
        R: io::Read,
    {
        let buf = err_at!(IOError, Rope::from_reader(data))?;
        let window: Vec<u64> = buf.lines().map(|rs| rs.chars().len() as u64).collect();
        Ok(Buffer::Normal {
            buf,
            window,
            cursor: (1, 1),
        })
    }

    pub fn into_insert(self) -> Buffer {
        match self {
            Buffer::Normal {
                buf,
                window,
                cursor,
            } => Buffer::Insert {
                buf,
                window,
                cursor,
            },
            v @ Buffer::Insert { .. } => v,
            Buffer::Replace {
                buf,
                window,
                cursor,
            } => Buffer::Insert {
                buf,
                window,
                cursor,
            },
        }
    }

    pub fn into_replace(self) -> Buffer {
        match self {
            Buffer::Normal {
                buf,
                window,
                cursor,
            } => Buffer::Replace {
                buf,
                window,
                cursor,
            },
            Buffer::Insert {
                buf,
                window,
                cursor,
            } => Buffer::Replace {
                buf,
                window,
                cursor,
            },
            v @ Buffer::Replace { .. } => v,
        }
    }

    pub fn into_normal(self) -> Buffer {
        match self {
            v @ Buffer::Normal { .. } => v,
            Buffer::Insert {
                buf,
                window,
                cursor,
            } => Buffer::Normal {
                buf,
                window,
                cursor,
            },
            Buffer::Replace {
                buf,
                window,
                cursor,
            } => Buffer::Normal {
                buf,
                window,
                cursor,
            },
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
}

impl Buffer {
    pub fn handle_event(&mut self, _event: &Event) -> Result<()> {
        todo!()
    }
}
