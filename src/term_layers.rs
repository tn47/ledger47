use crossterm::{cursor, style, terminal, Command};
use unicode_width::UnicodeWidthStr;

use crate::term_elements as te;
use ledger::core::{Error, Result};

pub struct LayerNewWorkspace {
    coord: te::Coordinates,
}

impl LayerNewWorkspace {
    pub fn new(coord: te::Coordinates) -> Result<LayerNewWorkspace> {
        Ok(LayerNewWorkspace { coord })
    }

    pub fn to_url() -> String {
        "/workspace/new".to_string()
    }
}

impl Command for LayerNewWorkspace {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        let mut title = {
            let content = "create new workspace".to_string();
            let c = self.coord.to_coord(2, 0, 1, (content.width() as u16) + 2);
            let mut t = te::Title::new(c, &content).ok().unwrap();
            t.on(te::BgLayer).with(te::FgTitle);
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
            b.on(te::BgLayer).with(te::FgBorder);
            b
        };

        let (col, row) = self.coord.to_origin();
        let mut output: String = Default::default();

        output.push_str(&cursor::MoveTo(col, row).to_string());
        output.push_str(&style::SetBackgroundColor(te::BgLayer).to_string());
        output.push_str(&terminal::Clear(terminal::ClearType::All).to_string());
        output.push_str(&border.to_string());
        output.push_str(&title.to_string());
        output.push_str(&cursor::Hide.to_string());

        output
    }
}
