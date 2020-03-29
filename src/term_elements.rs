use crossterm::{
    cursor,
    style::{self, Attribute, Color},
    Command,
};
use unicode_width::UnicodeWidthChar;

use std::iter::FromIterator;

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
        assert!(g_width < coord.width);
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
        let mut output: String = Default::default();
        {
            let (col, row) = self.coord.origin;
            output.push_str(&cursor::MoveTo(self.start, row).to_string());
        }
        {
            let s = style::style(self.buffer.to_string())
                .on(self.bg)
                .with(self.fg)
                .attribute(self.attr);
            output.push_str(&s.to_string());
        }
        output
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
        use std::iter::repeat;

        let mut output: String = Default::default();
        let (col, row) = self.coord.origin;
        let (ht, wd) = (self.coord.height, self.coord.width);

        // top
        {
            output.push_str(&cursor::MoveTo(col, row).to_string());
            output.push_str({
                let s = String::from_iter(repeat('─').take(wd as usize));
                &style::style(s)
                    .on(self.bg)
                    .with(self.fg)
                    .attribute(self.attr)
                    .to_string()
            });
        };
        // right
        {
            for h in 0..ht {
                output.push_str(&cursor::MoveTo(col + wd - 1, row + h).to_string());
                output.push_str(
                    &style::style('│')
                        .on(self.bg)
                        .with(self.fg)
                        .attribute(self.attr)
                        .to_string(),
                );
            }
        };
        // botton
        {
            output.push_str(&cursor::MoveTo(col, row + ht - 1).to_string());
            output.push_str({
                let s = String::from_iter(repeat('─').take(wd as usize));
                &style::style(s)
                    .on(self.bg)
                    .with(self.fg)
                    .attribute(self.attr)
                    .to_string()
            });
        };
        // left
        {
            for h in 0..ht {
                output.push_str(&cursor::MoveTo(col, row + h).to_string());
                output.push_str(
                    &style::style('│')
                        .on(self.bg)
                        .with(self.fg)
                        .attribute(self.attr)
                        .to_string(),
                );
            }
        };
        // top-left corner
        output.push_str(&cursor::MoveTo(col, row).to_string());
        output.push_str(
            &style::style('╭')
                .on(self.bg)
                .with(self.fg)
                .attribute(self.attr)
                .to_string(),
        );
        // top-right corner
        output.push_str(&cursor::MoveTo(col + wd - 1, row).to_string());
        output.push_str(
            &style::style('╮')
                .on(self.bg)
                .with(self.fg)
                .attribute(self.attr)
                .to_string(),
        );
        // bottom-right corner
        output.push_str(&cursor::MoveTo(col + wd - 1, row + ht - 1).to_string());
        output.push_str(
            &style::style('╯')
                .on(self.bg)
                .with(self.fg)
                .attribute(self.attr)
                .to_string(),
        );
        // bottom-left corner
        output.push_str(&cursor::MoveTo(col, row + ht - 1).to_string());
        output.push_str(
            &style::style('╰')
                .on(self.bg)
                .with(self.fg)
                .attribute(self.attr)
                .to_string(),
        );

        output
    }
}
