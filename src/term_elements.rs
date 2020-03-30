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
pub const BgInput: Color = Color::AnsiValue(234);
pub const FgInput: Color = Color::AnsiValue(15);
pub const FgInputName: Color = Color::AnsiValue(15);
pub const FgSection: Color = Color::AnsiValue(11);

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

#[derive(Default, Clone)]
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

#[derive(Default, Clone)]
pub struct Title {
    coord: Coordinates,
    content: String,
}

impl_command!(Title);

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

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        let (col, row) = self.coord.origin;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.content.to_string())
                .on(BgLayer)
                .with(FgTitle)
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
        assert!(coord.height > 2);
        assert!(coord.width > 2);
        Ok(Border { coord })
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
        assert!(n_prefix < coord.width, "{}/{}", n_prefix, coord.width);
        assert!(coord.height == 1);

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

        let (col, row) = self.coord.origin;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.prefix.clone())
                .on(BgLayer)
                .with(FgInputName)
        );
        write!(
            f,
            "{}",
            style::style(String::from_iter(
                repeat(' ').take((self.coord.width - self.n_prefix) as usize)
            ))
            .on(BgInput)
            .with(FgInput)
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
        let width = content
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;
        assert!(width < coord.width, "{}/{}", width, coord.width);
        assert!(coord.height == 1);

        Ok(TextLine {
            coord,
            content: content.to_string(),
            fg,
        })
    }
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use std::iter::repeat;

        let (col, row) = self.coord.origin;
        write!(f, "{}", cursor::MoveTo(col, row).to_string())?;
        write!(
            f,
            "{}",
            style::style(self.content.clone()).on(BgLayer).with(self.fg)
        )
    }
}
