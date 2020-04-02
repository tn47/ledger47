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
        let coord = te::Coordinates::new(te::Viewport::new(0, 0, view.tm.rows, view.tm.cols));
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
            let mut vp = self.coord.to_viewport();
            vp.move_by(2, 0).resize(1, (content.width() as u16) + 2);
            te::Title::new(te::Coordinates::new(vp), &content)
                .ok()
                .unwrap()
        };
        self.border = {
            let (height, width) = self.coord.to_viewport().to_size();
            let coord = te::Coordinates::new(te::Viewport::new(0, 0, height - 1, width));
            te::Border::new(coord).ok().unwrap()
        };
        self.ws_input_name = {
            let prefix = "Enter workspace name :";
            let coord = te::Coordinates::new(te::Viewport::new(3, 4, 1, 60));
            te::InputLine::new(coord, prefix).ok().unwrap()
        };
        self.comm_head = {
            let content = "Enter default commodity details";
            let coord = te::Coordinates::new(te::Viewport::new(3, 6, 1, 60));
            te::TextLine::new(coord, content, te::FgSection)
                .ok()
                .unwrap()
        };
        self.comm_input_name = {
            let prefix = "name :";
            let coord = te::Coordinates::new(te::Viewport::new(6, 8, 1, 40));
            te::InputLine::new(coord, prefix).ok().unwrap()
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
        let (col, row) = self.coord.to_viewport().to_origin();
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
