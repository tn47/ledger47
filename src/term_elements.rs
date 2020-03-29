use crossterm::{
    cursor,
    style::{self, Attribute, Color},
    Command,
};
use unicode_width::UnicodeWidthChar;

use std::{fmt, iter::FromIterator, result};

use crate::term_buffer::Buffer;
use ledger::core::{Error, Result};

//#[macro_export]
//macro_rules! element_style {
//    {$t:ty} => {
//        impl Command for $t {
//            type AnsiType = String;
//
//            fn ansi_code(&self) -> Self::AnsiType {
//                let mut output: String = Default::default();
//                for (col, row, item) in self.style.render(&self.buffer, self.to_view()) {
//                    output.push_str(&cursor::MoveTo(col, row).to_string());
//                    output.push_str(&item.to_string());
//                }
//                output
//            }
//        }
//    };
//}

// element_style! {Title<E>}

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
    buffer: Buffer,
    start: u16,
    g_width: u16,
    bg: Color,
    fg: Color,
    attr: Attribute,
}

impl Title {
    pub fn new(coord: Coordinates, content: &str) -> Result<Title> {
        assert!(coord.height == 1);
        let buffer = " ".to_string() + content + " ";
        let g_width = buffer
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>() as u16;
        assert!(g_width <= coord.width, "{} {}", g_width, coord.width);
        Ok(Title {
            coord: coord.clone(),
            buffer: Buffer::new(buffer.as_bytes())?,
            start: coord.origin.0,
            g_width,
            bg: Color::Black,
            fg: Color::White,
            attr: Attribute::Bold,
        })
    }

    pub fn align(&mut self, how: &str, fill: String) -> Result<&mut Self> {
        self.start = match how {
            "left" => self.start,
            "right" => self.start + self.coord.width - self.g_width,
            "center" | "middle" => self.start + ((self.coord.width - self.g_width) / 2),
            _ => err_at!(Fatal, msg: format!("unreachable"))?,
        };

        Ok(self)
    }

    pub fn on(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }

    pub fn with(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    pub fn attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attr = attr;
        self
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
        write!(f, "{}", cursor::MoveTo(self.start, row).to_string())?;

        let s = style::style(self.buffer.to_string())
            .on(self.bg)
            .with(self.fg)
            .attribute(self.attr);
        write!(f, "{}", s.to_string())
    }
}

pub struct Border {
    coord: Coordinates,
    bg: Color,
    fg: Color,
    attr: Attribute,
}

impl Border {
    pub fn new(coord: Coordinates) -> Result<Border> {
        assert!(coord.height > 2);
        assert!(coord.width > 2);
        Ok(Border {
            coord,
            bg: Color::Black,
            fg: Color::Black,
            attr: Attribute::Bold,
        })
    }

    pub fn on(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }

    pub fn with(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    pub fn attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attr = attr;
        self
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
        write!(f, "{}", style::SetBackgroundColor(self.bg).to_string())?;
        write!(f, "{}", style::SetForegroundColor(self.fg).to_string())?;
        write!(f, "{}", style::SetAttribute(self.attr).to_string())?;

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
