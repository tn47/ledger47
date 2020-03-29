use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent},
    execute,
    style::{self, Color},
    terminal, Command,
};
use unicode_width::UnicodeWidthStr;

use std::io::{self, Write};

use crate::term_elements as te;
use ledger::core::{Error, Result};

const MIN_COL: u64 = 1;
const MIN_ROW: u64 = 1;

const BgPage: Color = Color::AnsiValue(236);
const FgTitle: Color = Color::AnsiValue(6);
const FgBorder: Color = Color::AnsiValue(15);

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

pub fn run() -> Result<()> {
    let mut tm = Terminal::init()?;

    let page = PageNewWorkspace::new(te::Coordinates::new(0, 0, tm.rows, tm.cols))?;
    err_at!(Fatal, execute!(tm.stdout, page))?;

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

//struct Application {
//    cols: u16,
//    rows: u16,
//    pages: Vec<Box<dyn Page>>,
//}

struct PageNewWorkspace {
    coord: te::Coordinates,
}

impl PageNewWorkspace {
    fn new(coord: te::Coordinates) -> Result<PageNewWorkspace> {
        Ok(PageNewWorkspace { coord })
    }

    fn to_url() -> String {
        "/workspace/new".to_string()
    }
}

impl Command for PageNewWorkspace {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        let mut title = {
            let content = "create new workspace".to_string();
            let c = self.coord.to_coord(2, 0, 1, (content.width() as u16) + 2);
            let mut t = te::Title::new(c, &content).ok().unwrap();
            t.on(BgPage).with(FgTitle);
            t
        };
        let mut border = {
            let mut b = te::Border::new(te::Coordinates::new(
                0,
                0,
                self.coord.to_height() - 1,
                self.coord.to_width(),
            ))
            .ok()
            .unwrap();
            b.on(BgPage).with(FgBorder);
            b
        };

        let (col, row) = self.coord.to_origin();
        let mut output: String = Default::default();

        output.push_str(&cursor::MoveTo(col, row).to_string());
        output.push_str(&style::SetBackgroundColor(BgPage).to_string());
        output.push_str(&terminal::Clear(terminal::ClearType::All).to_string());
        output.push_str(&border.to_string());
        output.push_str(&title.to_string());
        output.push_str(&cursor::Hide.to_string());

        output
    }
}
