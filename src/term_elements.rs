use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    style::{self, Color},
    Command as TermCommand,
};
use log::{debug, trace};
use unicode_width::UnicodeWidthChar;

use std::{
    cmp,
    convert::TryInto,
    fmt,
    iter::FromIterator,
    ops::{self, RangeBounds},
    result,
};

use crate::{
    app::Application,
    edit_buffer::{Buffer, EditRes},
    util,
};
use ledger::core::{Result, Store};

pub const MIN_COL: u64 = 1;
pub const MIN_ROW: u64 = 1;

pub const BG_LAYER: Color = Color::AnsiValue(235);
pub const BG_EDIT: Color = Color::AnsiValue(232);

pub const FG_PERIOD: Color = Color::AnsiValue(27);
pub const FG_DATE: Color = Color::AnsiValue(33);
pub const FG_TITLE: Color = Color::AnsiValue(6);
pub const FG_BORDER: Color = Color::AnsiValue(15);
pub const FG_EDIT_INLINE: Color = Color::AnsiValue(59);
pub const FG_EDIT: Color = Color::AnsiValue(15);
pub const FG_SECTION: Color = Color::AnsiValue(11);
pub const FG_STATUS: Color = Color::AnsiValue(15);

macro_rules! impl_command {
    ($e:tt) => {
        impl TermCommand for $e {
            type AnsiType = String;

            fn ansi_code(&self) -> Self::AnsiType {
                self.to_string()
            }
        }
    };
}

pub enum Element {
    HeadLine(HeadLine),
    Title(Title),
    Border(Border),
    TextLine(TextLine),
    EditLine(EditLine),
    EditBox(EditBox),
    StatusLine(StatusLine),
}

impl Element {
    pub fn to_string(&self) -> String {
        match self {
            Element::HeadLine(em) => em.to_string(),
            Element::Title(em) => em.to_string(),
            Element::Border(em) => em.to_string(),
            Element::EditLine(em) => em.to_string(),
            Element::EditBox(em) => em.to_string(),
            Element::TextLine(em) => em.to_string(),
            Element::StatusLine(em) => em.to_string(),
        }
    }

    pub fn contain_cell(&self, col: u16, row: u16) -> bool {
        match self {
            Element::HeadLine(em) => em.vp.contain_cell(col, row),
            Element::Title(em) => em.vp.contain_cell(col, row),
            Element::Border(em) => em.vp.contain_cell(col, row),
            Element::EditLine(em) => em.vp.contain_cell(col, row),
            Element::EditBox(em) => em.vp.contain_cell(col, row),
            Element::TextLine(em) => em.vp.contain_cell(col, row),
            Element::StatusLine(em) => em.vp.contain_cell(col, row),
        }
    }

    pub fn refresh<S>(&mut self, app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        match self {
            Element::HeadLine(em) => em.refresh(app),
            Element::Title(em) => em.refresh(app),
            Element::Border(em) => em.refresh(app),
            Element::EditLine(em) => em.refresh(app),
            Element::EditBox(em) => em.refresh(app),
            Element::TextLine(em) => em.refresh(app),
            Element::StatusLine(em) => em.refresh(app),
        }
    }

    pub fn focus<S>(&mut self, app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        match self {
            Element::HeadLine(em) => em.focus(app),
            Element::Title(em) => em.focus(app),
            Element::Border(em) => em.focus(app),
            Element::EditLine(em) => em.focus(app),
            Element::EditBox(em) => em.focus(app),
            Element::TextLine(em) => em.focus(app),
            Element::StatusLine(em) => em.focus(app),
        }
    }

    pub fn leave<S>(&mut self, app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        match self {
            Element::HeadLine(em) => em.leave(app),
            Element::Title(em) => em.leave(app),
            Element::Border(em) => em.leave(app),
            Element::EditLine(em) => em.leave(app),
            Element::EditBox(em) => em.leave(app),
            Element::TextLine(em) => em.leave(app),
            Element::StatusLine(em) => em.leave(app),
        }
    }

    pub fn handle_event<S>(
        &mut self,
        app: &mut Application<S>,
        evnt: Event,
    ) -> Result<Option<Event>>
    where
        S: Store,
    {
        match self {
            Element::HeadLine(em) => em.handle_event(app, evnt),
            Element::Title(em) => em.handle_event(app, evnt),
            Element::Border(em) => em.handle_event(app, evnt),
            Element::EditLine(em) => em.handle_event(app, evnt),
            Element::EditBox(em) => em.handle_event(app, evnt),
            Element::TextLine(em) => em.handle_event(app, evnt),
            Element::StatusLine(em) => em.handle_event(app, evnt),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Viewport {
    col: u16,
    row: u16,
    height: u16,
    width: u16,
    ed_origin: (usize, usize), // absolute (col, row) within buffer, (0,0)
    vp_cursor_off: (u16, u16), // (col-offset, row-offset)
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
            ed_origin: Default::default(),
            vp_cursor_off: Default::default(),
        }
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
    pub fn to_ed_origin(&self) -> (usize, usize) {
        self.ed_origin
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

    pub fn to_cursor_off(&self) -> (u16, u16) {
        self.vp_cursor_off
    }

    pub fn apply_ed_cursor(&mut self, ed_cursor: (usize, usize)) {
        let (cdiff, rdiff) = match (self.to_ed_cursor(self.ed_origin), ed_cursor) {
            ((old_c, old_r), (new_c, new_r)) => (
                (new_c as isize) - (old_c as isize),
                (new_r as isize) - (old_r as isize),
            ),
        };

        let ccol = ((self.col + self.vp_cursor_off.0) as isize) + cdiff;
        let crow = ((self.row + self.vp_cursor_off.1) as isize) + rdiff;

        let (vp_col, ed_col): (u16, usize) = if ccol < (self.to_left() as isize) {
            (0, ed_cursor.0)
        } else if ccol > (self.to_right() as isize) {
            (self.width - 1, ed_cursor.0 - (self.width as usize) + 1)
        } else {
            let new_col: u16 = ccol.try_into().unwrap();
            trace!("{} {} {}", ccol, new_col, self.col);
            (new_col - self.col, self.ed_origin.0)
        };
        let (vp_row, ed_row): (u16, usize) = if crow < (self.to_top() as isize) {
            (0, ed_cursor.1)
        } else if crow > (self.to_bottom() as isize) {
            (self.height - 1, ed_cursor.1 - (self.height as usize) + 1)
        } else {
            let new_row: u16 = crow.try_into().unwrap();
            (new_row - self.row, self.ed_origin.1)
        };

        trace!(
            "ed_cursor:{:?} ed_origin:{:?}->{:?} vp_cursor:{:?}->{:?}",
            ed_cursor,
            self.ed_origin,
            (ed_col, ed_row),
            self.vp_cursor_off,
            (vp_col, vp_row)
        );

        self.ed_origin = (ed_col, ed_row);
        self.vp_cursor_off = (vp_col, vp_row);
    }

    fn to_ed_cursor(&self, ed_origin: (usize, usize)) -> (usize, usize) {
        let col = ed_origin.0 + (self.vp_cursor_off.0 as usize);
        let row = ed_origin.1 + (self.vp_cursor_off.1 as usize);
        (col, row)
    }
}

#[derive(Clone)]
pub struct HeadLine {
    vp: Viewport,
    date: chrono::Date<chrono::Local>,
    period: (chrono::Date<chrono::Local>, chrono::Date<chrono::Local>),
}

impl Default for HeadLine {
    fn default() -> Self {
        HeadLine {
            vp: Default::default(),
            date: chrono::Local::now().date(),
            period: util::date_to_period(chrono::Local::now().date()),
        }
    }
}

impl_command!(HeadLine);

impl HeadLine {
    pub fn new<S>(vp: Viewport, app: &Application<S>) -> Result<HeadLine>
    where
        S: Store,
    {
        let date = app.to_local_date();
        let period = app.to_local_period();

        Ok(HeadLine { vp, date, period })
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus headline");
        Ok(())
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for HeadLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "HeadLine::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

        let s_date = self.date.format("%d-%b-%y").to_string();
        let ss_date = style::style(&s_date).on(BG_LAYER).with(FG_DATE);
        let s_per0 = self.period.0.format("%d-%b-%y").to_string();
        let ss_per0 = style::style(&s_per0).on(BG_LAYER).with(FG_PERIOD);
        let s_per1 = self.period.1.format("%d-%b-%y").to_string();
        let ss_per1 = style::style(&s_per1).on(BG_LAYER).with(FG_PERIOD);

        let mut line = {
            let n = (width as usize) - s_per0.len() - s_per1.len() - s_date.len() - 3;
            style::style(&String::from_iter(repeat(' ').take(n)))
                .on(BG_LAYER)
                .to_string()
        };
        line.push_str(&format!(
            "{}{}{}{}{}",
            ss_per0,
            style::style("..").on(BG_LAYER).with(FG_BORDER),
            ss_per1,
            style::style(" ").on(BG_LAYER).with(FG_BORDER),
            ss_date
        ));

        write!(f, "{}", cursor::MoveTo(col - 1, row - 1).to_string())?;
        write!(f, "{}", line)
    }
}

#[derive(Clone)]
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

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus title");
        Ok(())
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "Title::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

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

#[derive(Clone)]
pub struct Border {
    vp: Viewport,
}

impl_command!(Border);

impl Border {
    pub fn new(vp: Viewport) -> Result<Border> {
        Ok(Border { vp })
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus border");
        Ok(())
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for Border {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (ht, wd) = self.vp.to_size();

        trace!(
            "Border::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            ht,
            wd
        );
        let (col, row) = (col - 1, row - 1);

        write!(f, "{}", style::SetBackgroundColor(BG_LAYER).to_string())?;
        write!(f, "{}", style::SetForegroundColor(FG_BORDER).to_string())?;

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

impl TextLine {
    pub fn new(vp: Viewport, content: &str, fg: Color) -> Result<TextLine> {
        Ok(TextLine {
            vp,
            content: content.to_string(),
            fg,
        })
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus text-line");
        Ok(())
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "TextLine::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

        write!(f, "{}", cursor::MoveTo(col - 1, row - 1).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.content.clone())
                .on(BG_LAYER)
                .with(self.fg)
        )
    }
}

#[derive(Clone, Default)]
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
        use std::iter::repeat;

        if msg.len() > 0 {
            debug!("Status <- {}", msg);
        }

        let (_, width) = self.vp.to_size();
        self.line = msg.to_string();
        if self.line.len() < (width as usize) {
            let n = (width as usize) - self.line.len();
            self.line += &String::from_iter(repeat(' ').take(n));
        }
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus status-line");
        Ok(())
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        Ok(Some(evnt))
    }
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "StatusLine::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

        write!(f, "{}", cursor::MoveTo(col - 1, row - 1).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.line.clone()).on(BG_LAYER).with(FG_STATUS)
        )
    }
}

#[derive(Clone)]
pub struct EditLine {
    vp: Viewport,
    inline: String,
    buffer: Buffer,
}

impl_command!(EditLine);

impl EditLine {
    pub fn new(vp: Viewport, inline: &str) -> Result<EditLine> {
        Ok(EditLine {
            vp,
            inline: inline.to_string(),
            buffer: Buffer::empty()?.change_to_insert(),
        })
    }

    pub fn clear_inline(&mut self) -> Result<()> {
        self.inline.clear();
        Ok(())
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        let (col, row) = match (self.vp.to_origin(), self.vp.to_cursor_off()) {
            ((col, row), (c, r)) => (col + c, row + r),
        };
        trace!(
            "Focus edit-line {:?} {:?}",
            self.vp.to_origin(),
            self.vp.to_cursor_off()
        );
        app.show_cursor_at(col, row)
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        match (to_modifiers(&evnt), to_key_code(&evnt)) {
            (_, Some(KeyCode::Enter))
            | (_, Some(KeyCode::Up))
            | (_, Some(KeyCode::Down))
            | (_, Some(KeyCode::PageUp))
            | (_, Some(KeyCode::PageDown))
            | (_, Some(KeyCode::Tab)) => Ok(Some(evnt)),
            (m, Some(KeyCode::BackTab)) if m.is_empty() => Ok(Some(evnt)),
            _ => match self.buffer.handle_event(evnt)? {
                EditRes {
                    col_at,
                    row_at,
                    evnt,
                } => {
                    self.vp.apply_ed_cursor((col_at, row_at));
                    Ok(evnt)
                }
            },
        }
    }
}

impl fmt::Display for EditLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "EditLine::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );
        let (col, row) = (col - 1, row - 1);

        let view_line = String::from_iter(repeat(' ').take(width as usize));
        let inline = {
            let n_inline = cmp::min(
                self.inline
                    .chars()
                    .map(|c| c.width().unwrap_or(0))
                    .sum::<usize>() as u16,
                width,
            ) as usize;
            String::from_iter(self.inline.chars().take(n_inline))
        };
        let buf_line = {
            let (ed_col, _ed_row) = self.vp.to_ed_origin();
            let mut lines = self
                .buffer
                .to_lines(0, 1)
                .into_iter()
                .map(|s| {
                    s.chars()
                        .skip(ed_col)
                        .take(width as usize)
                        .collect::<Vec<char>>()
                })
                .collect::<Vec<Vec<char>>>();
            String::from_iter(match lines.len() {
                0 => vec![].into_iter(),
                _ => lines.remove(0).into_iter(),
            })
        };

        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(f, "{}", style::style(view_line).on(BG_EDIT).with(BG_EDIT))?;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(inline).on(BG_EDIT).with(FG_EDIT_INLINE)
        )?;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(f, "{}", style::style(buf_line).on(BG_EDIT).with(FG_EDIT))
    }
}

#[derive(Clone)]
pub struct EditBox {
    vp: Viewport,
    inline: String,
    buffer: Buffer,
}

impl_command!(EditBox);

impl EditBox {
    pub fn new(vp: Viewport, inline: &str) -> Result<EditBox> {
        Ok(EditBox {
            vp,
            inline: inline.to_string(),
            buffer: Buffer::empty()?.change_to_insert(),
        })
    }

    pub fn clear_inline(&mut self) -> Result<()> {
        self.inline.clear();
        Ok(())
    }

    fn refresh<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn focus<S>(&mut self, app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        trace!("Focus edit-box");
        let (col, row) = match (self.vp.to_origin(), self.vp.to_cursor_off()) {
            ((col, row), (c, r)) => (col + c, row + r),
        };
        app.show_cursor_at(col, row)
    }

    fn leave<S>(&mut self, _app: &mut Application<S>) -> Result<()>
    where
        S: Store,
    {
        Ok(())
    }

    fn handle_event<S>(&mut self, _app: &mut Application<S>, evnt: Event) -> Result<Option<Event>>
    where
        S: Store,
    {
        match evnt {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => Ok(Some(evnt)),
            evnt => {
                let er = self.buffer.handle_event(evnt)?;
                Ok(er.evnt)
            }
        }
    }
}

impl fmt::Display for EditBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "EditBox::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

        let (col, row) = (col - 1, row - 1);

        let view_line = String::from_iter(repeat(' ').take(width as usize));
        let inline = {
            let n_inline = cmp::min(
                self.inline
                    .chars()
                    .map(|c| c.width().unwrap_or(0))
                    .sum::<usize>() as u16,
                width,
            ) as usize;
            String::from_iter(self.inline.chars().take(n_inline))
        };

        for i in 0..height {
            write!(f, "{}", cursor::MoveTo(col, row + (i as u16)).to_string())?;
            write!(
                f,
                "{}",
                style::style(view_line.clone()).on(BG_EDIT).with(BG_EDIT)
            )?;
        }
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(inline).on(BG_EDIT).with(FG_EDIT_INLINE)
        )?;
        let (from, till) = self.vp.to_ed_origin();
        for (i, line) in self.buffer.to_lines(from, till + 1).into_iter().enumerate() {
            write!(f, "{}", cursor::MoveTo(col, row + (i as u16)).to_string())?;
            write!(
                f,
                "{}",
                style::style(line.to_string()).on(BG_EDIT).with(FG_EDIT)
            )?;
        }

        Ok(())
    }
}

pub fn to_modifiers(evnt: &Event) -> KeyModifiers {
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

pub fn to_key_code(evnt: &Event) -> Option<KeyCode> {
    match evnt {
        Event::Resize(_, _) => None,
        Event::Key(KeyEvent { code, .. }) => Some(code.clone()),
        Event::Mouse(_) => None,
    }
}
