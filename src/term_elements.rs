use crossterm::{
    cursor,
    style::{self, Color},
    Command,
};
use unicode_width::UnicodeWidthChar;

use std::{
    fmt,
    iter::FromIterator,
    ops::{self, RangeBounds},
    result,
};

use crate::term_buffer::Buffer;
use ledger::core::Result;

pub const MIN_COL: u64 = 1;
pub const MIN_ROW: u64 = 1;

pub const BG_LAYER: Color = Color::AnsiValue(236);
pub const FG_TITLE: Color = Color::AnsiValue(6);
pub const FG_BORDER: Color = Color::AnsiValue(15);
pub const BG_INPUT: Color = Color::AnsiValue(234);
pub const FG_INPUT: Color = Color::AnsiValue(15);
pub const FG_INPUT_NAME: Color = Color::AnsiValue(15);
pub const FG_SECTION: Color = Color::AnsiValue(11);

#[macro_export]
macro_rules! impl_command {
    ($e:ty) => {
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
            Element::Title(em) => em.coord.to_viewport().contain_cell(col, row),
            Element::Border(em) => em.coord.to_viewport().contain_cell(col, row),
            Element::InputLine(em) => em.coord.to_viewport().contain_cell(col, row),
            Element::TextLine(em) => em.coord.to_viewport().contain_cell(col, row),
        }
    }
}

#[derive(Clone, Default)]
pub struct Viewport(u16, u16, u16, u16); // (col, row, height, width)

impl Viewport {
    #[inline]
    pub fn new(col: u16, row: u16, height: u16, width: u16) -> Viewport {
        Viewport(col, row, height, width)
    }

    #[inline]
    pub fn col_range(&self) -> impl ops::RangeBounds<u16> {
        self.0..(self.0 + self.3)
    }

    #[inline]
    pub fn row_range(&self) -> impl ops::RangeBounds<u16> {
        self.1..(self.1 + self.2)
    }

    #[inline]
    pub fn contain_cell(&self, col: u16, row: u16) -> bool {
        self.col_range().contains(&col) && self.row_range().contains(&row)
    }

    #[inline]
    pub fn to_origin(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    #[inline]
    pub fn to_size(&self) -> (u16, u16) {
        (self.2, self.3)
    }

    #[inline]
    pub fn to_top(&self) -> u16 {
        self.1
    }

    #[inline]
    pub fn to_right(&self) -> u16 {
        self.0 + self.3 - 1
    }

    #[inline]
    pub fn to_bottom(&self) -> u16 {
        self.1 + self.2 - 1
    }

    #[inline]
    pub fn to_left(&self) -> u16 {
        self.0
    }

    #[inline]
    pub fn move_to(mut self, col: u16, row: u16) -> Self {
        self.0 = col;
        self.1 = row;
        self
    }

    #[inline]
    pub fn move_by(mut self, col_off: i16, row_off: i16) -> Self {
        self.0 = ((self.0 as i16) + col_off) as u16;
        self.1 = ((self.1 as i16) + row_off) as u16;
        self
    }

    #[inline]
    pub fn resize_to(mut self, height: u16, width: u16) -> Self {
        self.2 = height;
        self.3 = width;
        self
    }

    #[inline]
    pub fn resize_by(mut self, height_off: i16, width_off: i16) -> Self {
        self.2 = ((self.2 as i16) + height_off) as u16;
        self.3 = ((self.3 as i16) + width_off) as u16;
        self
    }
}

#[derive(Default, Clone)]
pub struct Coordinates {
    vp: Viewport,
    scroll_off: u16,
    cursor: Option<(u16, u16)>, // (col-offset, row-offset)
}

impl Coordinates {
    pub fn new(vp: Viewport) -> Coordinates {
        Coordinates {
            vp: vp.clone(),
            scroll_off: Default::default(),
            cursor: Some(vp.to_origin()),
        }
    }

    pub fn set_scroll_off(mut self, scroll_off: u16) -> Self {
        self.scroll_off = scroll_off;
        self
    }

    pub fn to_viewport(&self) -> Viewport {
        self.vp.clone()
    }

    pub fn to_cursor(&self) -> Option<(u16, u16)> {
        self.cursor.clone()
    }

    pub fn cursor_move_to(&mut self, col: u16, row: u16) -> (u16, u16) {
        match self.cursor {
            Some((_, _)) => {
                let (ccol, rcol) = if self.vp.col_range().contains(&col) {
                    (col, 0)
                } else {
                    (self.vp.to_right(), col - self.vp.to_right())
                };
                let (crow, rrow) = if self.vp.row_range().contains(&row) {
                    (row, 0)
                } else {
                    let crow = self.vp.to_bottom() - self.scroll_off;
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
pub struct Title {
    coord: Coordinates,
    content: String,
}

impl_command!(Title);

impl Title {
    pub fn new(coord: Coordinates, content: &str) -> Result<Title> {
        let content = " ".to_string() + content + " ";
        Ok(Title { coord, content })
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.coord.to_viewport().to_origin();
        write!(f, "{}", cursor::Hide)?;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
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
    coord: Coordinates,
}

impl_command!(Border);

impl Border {
    pub fn new(coord: Coordinates) -> Result<Border> {
        Ok(Border { coord })
    }
}

impl fmt::Display for Border {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.coord.to_viewport().to_origin();
        let (ht, wd) = self.coord.to_viewport().to_size();
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

#[derive(Default, Clone)]
pub struct InputLine {
    coord: Coordinates,
    prefix: String,
    n_prefix: u16,
    buffer: Buffer,
}

impl_command!(InputLine);

impl InputLine {
    pub fn new(coord: Coordinates, prefix: &str) -> Result<InputLine> {
        let n_prefix = prefix
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;

        let bytes: Vec<u8> = vec![];
        let buffer = Buffer::new(bytes.as_slice()).ok().unwrap().into_insert();
        Ok(InputLine {
            coord,
            prefix: prefix.to_string(),
            n_prefix,
            buffer,
        })
    }
}

impl fmt::Display for InputLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.coord.to_viewport().to_origin();
        let (_, width) = self.coord.to_viewport().to_size();

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
                repeat(' ').take((width - self.n_prefix) as usize)
            ))
            .on(BG_INPUT)
            .with(FG_INPUT)
        )
    }
}

#[derive(Clone)]
pub struct TextLine {
    coord: Coordinates,
    content: String,
    fg: Color,
}

impl Default for TextLine {
    fn default() -> Self {
        TextLine {
            coord: Default::default(),
            content: Default::default(),
            fg: Color::White,
        }
    }
}

impl_command!(TextLine);

impl TextLine {
    pub fn new(coord: Coordinates, content: &str, fg: Color) -> Result<TextLine> {
        Ok(TextLine {
            coord,
            content: content.to_string(),
            fg,
        })
    }
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.coord.to_viewport().to_origin();
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

//#[derive(Clone)]
//pub struct StatusLine {
//    coord: Coordinates,
//    content: String,
//}
//
//impl Default for StatusLine {
//    fn default() -> Self {
//        StatusLine {
//            coord: Default::default(),
//            content: Default::default(),
//        }
//    }
//}
//
//impl_command!(StatusLine);
//
//impl StatusLine {
//    pub fn new(coord: Coordinates) -> Result<StatusLine> {
//        Ok(StatusLine {
//            coord,
//            content: Default::default(),
//        })
//    }
//}
//
//impl fmt::Display for TextLine {
//    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
//        use std::iter::repeat;
//
//        let (col, row) = self.coord.origin;
//        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
//        write!(
//            f,
//            "{}",
//            style::style(self.content.clone()).on(BG_LAYER).with(self.fg)
//        )
//    }
//}
