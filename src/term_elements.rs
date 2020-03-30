use crossterm::{
    cursor,
    style::{self, Attribute, Color},
    Command,
};
use unicode_width::UnicodeWidthChar;

use std::{fmt, iter::FromIterator, result};

use crate::app::Application;
use crate::term_buffer::Buffer;
use ledger::core::{Error, Result};

pub const MIN_COL: u64 = 1;
pub const MIN_ROW: u64 = 1;

pub const BgLayer: Color = Color::AnsiValue(236);
pub const FgTitle: Color = Color::AnsiValue(6);
pub const FgBorder: Color = Color::AnsiValue(15);

#[derive(Clone)]
pub struct Coordinates {
    origin: (u16, u16),
    height: u16,
    width: u16,
}

impl Coordinates {
    pub fn new(col: u16, row: u16, height: u16, width: u16) -> Coordinates {
        Coordinates {
            origin: (col, row),
            height,
            width,
        }
    }

    pub fn to_coord(&self, col_off: u16, row_off: u16, height: u16, width: u16) -> Coordinates {
        let (col, row) = self.origin;
        Coordinates {
            origin: (col + col_off, row + row_off),
            height,
            width,
        }
    }

    pub fn to_origin(&self) -> (u16, u16) {
        self.origin
    }

    pub fn to_height(&self) -> u16 {
        self.height
    }

    pub fn to_width(&self) -> u16 {
        self.width
    }
}

pub struct Title {
    coord: Coordinates,
    content: String,
}

impl Title {
    pub fn new(coord: Coordinates, content: &str) -> Result<Title> {
        assert!(coord.height == 1);
        let content = " ".to_string() + content + " ";
        let width = content
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;
        assert!(width <= coord.width, "{}/{}", width, coord.width);
        Ok(Title { coord, content })
    }
}

impl Title {
    fn build(&self) {
        ()
    }
}

impl Command for Title {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        self.to_string()
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.coord.origin;
        write!(
            f,
            "{}",
            cursor::MoveTo(self.coord.origin.0, row).to_string()
        )?;
        write!(
            f,
            "{}",
            style::style(self.content.to_string())
                .on(BgLayer)
                .with(FgTitle)
        )
    }
}

pub struct Border {
    coord: Coordinates,
}

impl Border {
    pub fn new(coord: Coordinates) -> Result<Border> {
        assert!(coord.height > 2);
        assert!(coord.width > 2);
        Ok(Border { coord })
    }
}

impl Command for Border {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        self.to_string()
    }
}

impl fmt::Display for Border {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.coord.origin;
        let (ht, wd) = (self.coord.height, self.coord.width);
        write!(f, "{}", style::SetBackgroundColor(BgLayer).to_string())?;
        write!(f, "{}", style::SetForegroundColor(FgBorder).to_string())?;

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
