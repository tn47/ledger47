use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent},
    execute,
    style::{self, Color},
    terminal,
};
use unicode_width::UnicodeWidthStr;

use std::io::{self, Write};

use crate::term_elements as te;
use crate::term_layers::{self as tl, Layer};
use ledger::core::{Error, Result};

pub struct Application<D> {
    layers: Vec<Layer>,
    store: D,
}

pub fn run() -> Result<()> {
    let mut tm = Terminal::init()?;

    let layer = tl::NewWorkspace::new(te::Coordinates::new(0, 0, tm.rows, tm.cols))?;
    err_at!(Fatal, execute!(tm.stdout, layer))?;

    loop {
        let code = match err_at!(Fatal, event::read())? {
            Event::Key(event) => handle_key_event(event),
            Event::Mouse(event) => handle_mouse_event(event),
            Event::Resize(width, height) => handle_resize(width, height),
        };
        if code > 0 {
            break Ok(());
        }
    }
}

fn handle_key_event(event: KeyEvent) -> u32 {
    println!("event {:?}", event);
    match event.code {
        KeyCode::Char('q') => 1,
        _ => 0,
    }
}

fn handle_mouse_event(event: MouseEvent) -> u32 {
    println!("event {:?}", event);
    0
}

fn handle_resize(width: u16, height: u16) -> u32 {
    println!("w:{} h:{}", width, height);
    0
}

struct Terminal {
    stdout: io::Stdout,
    cols: u16,
    rows: u16,
}

impl Terminal {
    fn init() -> Result<Terminal> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode();
        err_at!(Fatal, execute!(stdout, EnableMouseCapture, cursor::Hide))?;

        let (cols, rows) = err_at!(Fatal, terminal::size())?;
        Ok(Terminal { stdout, cols, rows })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        execute!(self.stdout, DisableMouseCapture, cursor::Show);
        terminal::disable_raw_mode();
    }
}
