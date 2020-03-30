use crossterm::{cursor, style, terminal, Command};
use unicode_width::UnicodeWidthStr;

use std::ffi;

use crate::app::View;
use crate::event::Event;
use crate::term_elements::{self as te};
use ledger::core::{Error, Result};

pub enum Layer {
    NewWorkspace(NewWorkspace),
}

impl Layer {
    pub fn resize(self, view: &mut View) -> Result<Self> {
        match self {
            Layer::NewWorkspace(layer) => layer.resize(view),
        }
    }

    pub fn build(&mut self, view: &mut View) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.build(view),
        }
    }

    pub fn handle_event(&mut self, view: &mut View, evnt: Event) -> Result<Option<Event>> {
        match self {
            Layer::NewWorkspace(layer) => layer.handle_event(view, evnt),
        }
    }
}

impl Command for Layer {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        match self {
            Layer::NewWorkspace(layer) => layer.ansi_code(),
        }
    }
}

pub struct NewWorkspace {
    coord: te::Coordinates,
    title: te::Title,
    border: te::Border,
    ws_input_name: te::InputLine,
    comm_head: te::TextLine,
    comm_input_name: te::InputLine,
}

impl NewWorkspace {
    pub fn to_url() -> String {
        "/workspace/new".to_string()
    }

    pub fn new_layer(view: &mut View) -> Result<Layer> {
        let coord = te::Coordinates::new(0, 0, view.tm.rows, view.tm.cols);
        Ok(Layer::NewWorkspace(NewWorkspace {
            coord,
            title: Default::default(),
            border: Default::default(),
            ws_input_name: Default::default(),
            comm_head: Default::default(),
            comm_input_name: Default::default(),
        }))
    }

    pub fn resize(self, view: &mut View) -> Result<Layer> {
        NewWorkspace::new_layer(view)
    }

    pub fn build(&mut self, _view: &View) -> Result<()> {
        self.title = {
            let content = "Create new workspace".to_string();
            let c = self.coord.to_coord(2, 0, 1, (content.width() as u16) + 2);
            te::Title::new(c, &content).ok().unwrap()
        };
        self.border = {
            let c = te::Coordinates::new(0, 0, self.coord.to_height() - 1, self.coord.to_width());
            te::Border::new(c).ok().unwrap()
        };
        self.ws_input_name = {
            let prefix = "Enter workspace name ";
            let c = te::Coordinates::new(3, 4, 1, 60);
            te::InputLine::new(c, prefix).ok().unwrap()
        };
        self.comm_head = {
            let content = "Enter default commodity details";
            let c = te::Coordinates::new(3, 6, 1, 60);
            te::TextLine::new(c, content, te::FgSection).ok().unwrap()
        };
        self.comm_input_name = {
            let prefix = "name :";
            let c = te::Coordinates::new(6, 8, 1, 40);
            te::InputLine::new(c, prefix).ok().unwrap()
        };

        Ok(())
    }

    pub fn handle_event(&mut self, view: &mut View, evnt: Event) -> Result<Option<Event>> {
        Ok(Some(evnt))
    }
}

impl Command for NewWorkspace {
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        let (col, row) = self.coord.to_origin();
        let mut output: String = Default::default();

        output.push_str(&cursor::MoveTo(col, row).to_string());
        output.push_str(&style::SetBackgroundColor(te::BgLayer).to_string());
        output.push_str(&terminal::Clear(terminal::ClearType::All).to_string());
        output.push_str(&self.border.to_string());
        output.push_str(&self.title.to_string());
        output.push_str(&self.ws_input_name.to_string());
        output.push_str(&self.comm_head.to_string());
        output.push_str(&self.comm_input_name.to_string());
        output.push_str(&cursor::Hide.to_string());

        output
    }
}
