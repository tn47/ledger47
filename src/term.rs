use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent},
    execute,
    style::{self, Color},
    terminal,
};

use std::io::{self, Write};

use crate::term_elements as te;
use ledger::core::{Error, Result};

const MIN_COL: u64 = 1;
const MIN_ROW: u64 = 1;

const PageBg: Color = Color::AnsiValue(236);
const PageTitleFg: Color = Color::AnsiValue(6);
const BorderFg: Color = Color::AnsiValue(15);

#[derive(Default)]
struct Terminal {
    cols: u16,
    rows: u16,
}

pub fn run() -> Result<()> {
    let mut tm: Terminal = Default::default();

    terminal::enable_raw_mode();

    let (cols, rows) = err_at!(Fatal, terminal::size())?;
    tm.cols = cols;
    tm.rows = rows;

    let mut stdout = io::stdout();

    err_at!(Fatal, execute!(stdout, EnableMouseCapture))?;
    let mut title = {
        let coord = te::Coordinates::new(2, 0, 1, tm.cols - 4);
        let fill = style::style('─').on(PageBg).with(BorderFg);
        let mut t = te::Title::new(coord, "hello world, welcome")?;
        t.on(PageBg)
            .with(PageTitleFg)
            .align("right", fill.to_string());
        t
    };
    let mut border = {
        let coord = te::Coordinates::new(0, 0, tm.rows, tm.cols);
        let mut b = te::Border::new(coord)?;
        b.on(PageBg).with(BorderFg);
        b
    };
    err_at!(
        Fatal,
        execute!(
            stdout,
            style::SetBackgroundColor(PageBg),
            terminal::Clear(terminal::ClearType::All),
            border,
            title,
            cursor::Hide,
        )
    )?;

    loop {
        let code = match err_at!(Fatal, event::read())? {
            Event::Key(event) => handle_key_event(event),
            Event::Mouse(event) => handle_mouse_event(event),
            Event::Resize(width, height) => handle_resize(width, height),
        };
        if code > 0 {
            break;
        }
    }

    err_at!(Fatal, execute!(stdout, DisableMouseCapture))?;
    terminal::disable_raw_mode();
    Ok(())
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

//struct Application {
//    cols: u16,
//    rows: u16,
//    pages: Vec<Box<dyn Page>>,
//}

struct PageNewWorkspace {
    cols: u16,
    rows: u16,
}

impl PageNewWorkspace {
    fn to_url() -> String {
        "/workspace/new".to_string()
    }
}
