use crossterm::{
    cursor,
    event::KeyCode,
    style::{self, Color},
    Command,
};
use unicode_width::UnicodeWidthChar;

use std::{
    fmt,
    iter::FromIterator,
    ops::{self, RangeBounds},
    result,
    str::FromStr,
};

use crate::app::Application;
use crate::edit_buffer::Buffer;
use crate::event::Event;
use ledger::core::{Result, Store};

pub const MIN_COL: u64 = 1;
pub const MIN_ROW: u64 = 1;

pub const BG_LAYER: Color = Color::AnsiValue(236);
pub const FG_TITLE: Color = Color::AnsiValue(6);
pub const FG_BORDER: Color = Color::AnsiValue(15);
pub const BG_INPUT: Color = Color::AnsiValue(234);
pub const FG_INPUT: Color = Color::AnsiValue(15);
pub const FG_INPUT_NAME: Color = Color::AnsiValue(15);
pub const FG_SECTION: Color = Color::AnsiValue(11);
pub const FG_STATUS: Color = Color::AnsiValue(15);

macro_rules! impl_command {
    ($e:tt) => {
        impl Command for $e {
            type AnsiType = String;

            fn ansi_code(&self) -> Self::AnsiType {
                self.to_string()
            }
        }
    };
}

enum Element {
    Title(Title),
    Border(Border),
    InputLine(InputLine),
    TextLine(TextLine),
}

impl Element {
    fn contain_cell(&self, col: u16, row: u16) -> bool {
        match self {
            Element::Title(em) => em.vp.contain_cell(col, row),
            Element::Border(em) => em.vp.contain_cell(col, row),
            Element::InputLine(em) => em.vp.contain_cell(col, row),
            Element::TextLine(em) => em.vp.contain_cell(col, row),
        }
    }

    pub fn handle_event<D, T>(
        &mut self,
        app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        match self {
            Element::Title(em) => em.handle_event(app, evnt),
            Element::Border(em) => em.handle_event(app, evnt),
            Element::InputLine(em) => em.handle_event(app, evnt),
            Element::TextLine(em) => em.handle_event(app, evnt),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Viewport {
    col: u16,
    row: u16,
    height: u16,
    width: u16,
    scroll_off: u16,
    cursor: Option<(u16, u16)>, // (col-offset, row-offset)
}

impl fmt::Display for Viewport {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(
            f,
            "Viewport<col:{} row:{} height:{} width:{}>",
            self.col, self.row, self.height, self.width
        )
    }
}

impl Viewport {
    #[inline]
    pub fn new(col: u16, row: u16, height: u16, width: u16) -> Viewport {
        Viewport {
            col,
            row,
            height,
            width,
            scroll_off: Default::default(),
            cursor: None,
        }
    }

    #[inline]
    pub fn set_scroll_off(mut self, scroll_off: u16) -> Self {
        self.scroll_off = scroll_off;
        self
    }

    #[inline]
    pub fn move_to(mut self, col: u16, row: u16) -> Self {
        self.col = col;
        self.row = row;
        self
    }

    #[inline]
    pub fn move_by(mut self, col_off: i16, row_off: i16) -> Self {
        self.col = ((self.col as i16) + col_off) as u16;
        self.row = ((self.row as i16) + row_off) as u16;
        self
    }

    #[inline]
    pub fn resize_to(mut self, height: u16, width: u16) -> Self {
        self.height = height;
        self.width = width;
        self
    }

    #[inline]
    pub fn resize_by(mut self, height_off: i16, width_off: i16) -> Self {
        self.height = ((self.height as i16) + height_off) as u16;
        self.width = ((self.width as i16) + width_off) as u16;
        self
    }
}

impl Viewport {
    #[inline]
    pub fn col_range(&self) -> impl ops::RangeBounds<u16> {
        self.col..(self.col + self.width)
    }

    #[inline]
    pub fn row_range(&self) -> impl ops::RangeBounds<u16> {
        self.row..(self.row + self.height)
    }

    #[inline]
    pub fn contain_cell(&self, col: u16, row: u16) -> bool {
        self.col_range().contains(&col) && self.row_range().contains(&row)
    }

    #[inline]
    pub fn to_origin(&self) -> (u16, u16) {
        (self.col, self.row)
    }

    #[inline]
    pub fn to_size(&self) -> (u16, u16) {
        (self.height, self.width)
    }

    #[inline]
    pub fn to_top(&self) -> u16 {
        self.row
    }

    #[inline]
    pub fn to_right(&self) -> u16 {
        self.col + self.width - 1
    }

    #[inline]
    pub fn to_bottom(&self) -> u16 {
        self.row + self.height - 1
    }

    #[inline]
    pub fn to_left(&self) -> u16 {
        self.col
    }

    pub fn to_cursor(&self) -> Option<(u16, u16)> {
        self.cursor.clone()
    }

    pub fn cursor_move_to(&mut self, col: u16, row: u16) -> (u16, u16) {
        match self.cursor {
            Some((_, _)) => {
                let (ccol, rcol) = if self.col_range().contains(&col) {
                    (col, 0)
                } else {
                    (self.to_right(), col - self.to_right())
                };
                let (crow, rrow) = if self.row_range().contains(&row) {
                    (row, 0)
                } else {
                    let crow = self.to_bottom() - self.scroll_off;
                    (crow, row - crow)
                };
                self.cursor = Some((ccol, crow));
                (rcol, rrow)
            }
            None => (0, 0),
        }
    }

    pub fn cursor_move_by(&mut self, col: u16, row: u16) -> (u16, u16) {
        match self.cursor {
            Some((ccol, crow)) => self.cursor_move_to(ccol + col, crow + row),
            None => (0, 0),
        }
    }
}

#[derive(Default, Clone)]
pub struct HeadLine {
    vp: Viewport,
    line: String,
}

impl_command!(HeadLine);

impl HeadLine {
    pub fn new<D, T>(vp: Viewport, app: &Application<D, T>) -> Result<HeadLine>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        use std::iter::repeat;

        let s_date = app.to_local_date().format("%d-%b-%y");
        let s_per0 = app.to_local_period().0.format("%d-%b-%y");
        let s_per1 = app.to_local_period().1.format("%d-%b-%y");

        let part0 = format!("{}/{} {}", s_per0, s_per1, s_date);
        let mut line = {
            let (_, width) = vp.to_size();
            String::from_iter(repeat(' ').take((width as usize) - part0.len()))
        };
        line.push_str(&part0);

        Ok(HeadLine { vp, line })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for HeadLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.line.clone()).on(BG_LAYER).with(FG_STATUS)
        )
    }
}

#[derive(Default, Clone)]
pub struct Title {
    vp: Viewport,
    content: String,
}

impl_command!(Title);

impl Title {
    pub fn new(vp: Viewport, content: &str) -> Result<Title> {
        let content = " ".to_string() + content + " ";
        Ok(Title { vp, content })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        write!(f, "{}", cursor::Hide)?;
        write!(f, "{}", cursor::MoveTo(col - 1, row - 1).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.content.clone())
                .on(BG_LAYER)
                .with(FG_TITLE)
        )
    }
}

#[derive(Default, Clone)]
pub struct Border {
    vp: Viewport,
}

impl_command!(Border);

impl Border {
    pub fn new(vp: Viewport) -> Result<Border> {
        Ok(Border { vp })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for Border {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = {
            let (c, r) = self.vp.to_origin();
            (c - 1, r - 1)
        };
        let (ht, wd) = self.vp.to_size();
        write!(f, "{}", style::SetBackgroundColor(BG_LAYER).to_string())?;
        write!(f, "{}", style::SetForegroundColor(FG_BORDER).to_string())?;

        write!(f, "{}", cursor::Hide)?;

        // top
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(f, "{}", String::from_iter(repeat('─').take(wd as usize)))?;
        // right
        for h in 0..ht {
            write!(f, "{}", cursor::MoveTo(col + wd - 1, row + h).to_string())?;
            write!(f, "│")?;
        }
        // botton
        write!(f, "{}", cursor::MoveTo(col, row + ht - 1).to_string())?;
        write!(f, "{}", String::from_iter(repeat('─').take(wd as usize)))?;
        // left
        for h in 0..ht {
            write!(f, "{}", cursor::MoveTo(col, row + h).to_string())?;
            write!(f, "│")?;
        }
        // top-left corner
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(f, "╭")?;
        // top-right corner
        write!(f, "{}", cursor::MoveTo(col + wd - 1, row).to_string())?;
        write!(f, "╮")?;
        // bottom-right corner
        write!(
            f,
            "{}",
            cursor::MoveTo(col + wd - 1, row + ht - 1).to_string()
        )?;
        write!(f, "╯")?;
        // bottom-left corner
        write!(f, "{}", cursor::MoveTo(col, row + ht - 1).to_string())?;
        write!(f, "╰")
    }
}

#[derive(Clone)]
pub struct TextLine {
    vp: Viewport,
    content: String,
    fg: Color,
}

impl_command!(TextLine);

impl Default for TextLine {
    fn default() -> Self {
        TextLine {
            vp: Default::default(),
            content: Default::default(),
            fg: Color::White,
        }
    }
}

impl TextLine {
    pub fn new(vp: Viewport, content: &str, fg: Color) -> Result<TextLine> {
        Ok(TextLine {
            vp,
            content: content.to_string(),
            fg,
        })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.content.clone())
                .on(BG_LAYER)
                .with(self.fg)
        )
    }
}

#[derive(Default, Clone)]
pub struct StatusLine {
    vp: Viewport,
    line: String,
}

impl_command!(StatusLine);

impl StatusLine {
    pub fn new(vp: Viewport) -> Result<StatusLine> {
        use std::iter::repeat;

        let line = {
            let (_, width) = vp.to_size();
            String::from_iter(repeat(' ').take(width as usize))
        };
        Ok(StatusLine { vp, line })
    }

    pub fn log(&mut self, msg: &str) {
        let (_, width) = self.vp.to_size();
        let (mut w, w1) = (width - 11, (width - 11) as usize);
        let s = String::from_iter(msg.chars().rev().take_while(|ch| {
            w -= ch.width().unwrap() as u16;
            w > 0
        }));

        self.line = format!(
            "{:width$} {}",
            s,
            chrono::Local::now().format("%d-%b-%y"),
            width = w1,
        );
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.line.clone()).on(BG_LAYER).with(FG_STATUS)
        )
    }
}

#[derive(Default, Clone)]
pub struct InputLine {
    vp: Viewport,
    prefix: String,
    buffer: Buffer,
}

impl_command!(InputLine);

impl InputLine {
    pub fn new(vp: Viewport, prefix: &str) -> Result<InputLine> {
        let bytes: Vec<u8> = vec![];
        Ok(InputLine {
            vp,
            prefix: prefix.to_string(),
            buffer: Buffer::empty()?.change_to_insert(),
        })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        match evnt {
            Event::Key {
                code: KeyCode::Enter,
                ..
            } => Ok(Some(evnt)),
            evnt => {
                let er = self.buffer.handle_event(evnt)?;
                Ok(er.evnt)
            }
        }
    }
}

impl fmt::Display for InputLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (_, width) = self.vp.to_size();

        let n_prefix = self
            .prefix
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;

        write!(f, "{}", cursor::Hide)?;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.prefix.clone())
                .on(BG_LAYER)
                .with(FG_INPUT_NAME)
        )?;
        write!(
            f,
            "{}",
            style::style(String::from_iter(
                repeat(' ').take((width - n_prefix) as usize)
            ))
            .on(BG_INPUT)
            .with(FG_INPUT)
        )
    }
}

#[derive(Default, Clone)]
pub struct InputBox {
    vp: Viewport,
    prefix: String,
    buffer: Buffer,
}

impl_command!(InputBox);

impl InputBox {
    pub fn new(vp: Viewport, prefix: &str) -> Result<InputBox> {
        let mut buffer = Buffer::empty()?.change_to_insert();
        Ok(InputBox {
            vp,
            prefix: prefix.to_string(),
            buffer,
        })
    }

    fn handle_event<D, T>(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        D: Store<T>,
        T: ToString + FromStr,
    {
        match evnt {
            Event::Key {
                code: KeyCode::Enter,
                ..
            } => Ok(Some(evnt)),
            evnt => {
                let er = self.buffer.handle_event(evnt)?;
                Ok(er.evnt)
            }
        }
    }
}

impl fmt::Display for InputBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        let n_prefix = self
            .prefix
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;

        write!(f, "{}", cursor::Hide)?;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.prefix.clone())
                .on(BG_LAYER)
                .with(FG_INPUT_NAME)
        )?;

        let (col, width) = (col + n_prefix, width - n_prefix);
        for row in row..(row + height) {
            write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
            write!(
                f,
                "{}",
                style::style(String::from_iter(repeat(' ').take((width) as usize)))
                    .on(BG_INPUT)
                    .with(FG_INPUT)
            )?;
        }

        Ok(())
    }
}
