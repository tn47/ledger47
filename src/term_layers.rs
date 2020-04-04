use crossterm::{cursor, style, terminal, Command};
use unicode_width::UnicodeWidthStr;

use std::{marker, str::FromStr};

use crate::app::Application;
use crate::event::Event;
use crate::term_elements::{self as te};
use ledger::core::{Result, Store};

pub enum Layer<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    NewWorkspace(NewWorkspace<D, T>),
}

impl<D, T> Layer<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    pub fn resize(self, app: &mut Application<D, T>) -> Result<Self> {
        match self {
            Layer::NewWorkspace(layer) => layer.resize(app),
        }
    }

    pub fn build(&mut self, app: &mut Application<D, T>) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.build(app),
        }
    }

    pub fn handle_event(
        &mut self,
        app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>> {
        match self {
            Layer::NewWorkspace(layer) => layer.handle_event(app, evnt),
        }
    }
}

impl<D, T> Command for Layer<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        match self {
            Layer::NewWorkspace(layer) => layer.ansi_code(),
        }
    }
}

pub struct NewWorkspace<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    vp: te::Viewport,
    // elements: Vec<te::Element>,
    title: te::Title,
    border: te::Border,
    ws_input_name: te::InputLine,
    comm_head: te::TextLine,
    comm_input_name: te::InputLine,

    _phantom_d: marker::PhantomData<D>,
    _phantom_t: marker::PhantomData<T>,
}

impl<D, T> NewWorkspace<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    pub fn to_url() -> String {
        "/workspace/new".to_string()
    }

    pub fn new_layer(app: &mut Application<D, T>) -> Result<Layer<D, T>> {
        let vp = app.to_viewport();
        Ok(Layer::NewWorkspace(NewWorkspace {
            vp,
            // elements: Default::default(),
            title: Default::default(),
            border: Default::default(),
            ws_input_name: Default::default(),
            comm_head: Default::default(),
            comm_input_name: Default::default(),

            _phantom_d: marker::PhantomData,
            _phantom_t: marker::PhantomData,
        }))
    }

    pub fn resize(self, app: &mut Application<D, T>) -> Result<Layer<D, T>> {
        NewWorkspace::new_layer(app)
    }

    pub fn build(&mut self, app: &mut Application<D, T>) -> Result<()> {
        //let (head, coord) = {
        //    (_, width) = self.coord.to_viewport().to_size();
        //    let vp = self.coord.to_viewport().resize_to(1, width);
        //    te::HeadLine::new(te::Coordinates::new(vp)
        //};
        self.title = {
            let content = "Create new workspace".to_string();
            let vp = self
                .vp
                .clone()
                .move_by(2, 0)
                .resize_to(1, (content.width() as u16) + 2);
            te::Title::new(vp, &content).ok().unwrap()
        };
        self.border = { te::Border::new(self.vp.clone()).ok().unwrap() };
        self.ws_input_name = {
            let prefix = "Enter workspace name :";
            let vp = self.vp.clone().move_by(2, 3).resize_to(1, 60);
            te::InputLine::new(vp, prefix).ok().unwrap()
        };
        self.comm_head = {
            let content = "Enter default commodity details";
            let vp = self.vp.clone().move_by(2, 5).resize_to(1, 60);
            te::TextLine::new(vp, content, te::FG_SECTION).ok().unwrap()
        };
        self.comm_input_name = {
            let prefix = "Name :";
            let vp = self.vp.clone().move_by(5, 7).resize_to(1, 40);
            te::InputLine::new(vp, prefix).ok().unwrap()
        };

        Ok(())
    }

    pub fn handle_event(
        &mut self,
        _app: &mut Application<D, T>,
        evnt: Event,
    ) -> Result<Option<Event>> {
        Ok(Some(evnt))
    }
}

impl<D, T> Command for NewWorkspace<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        let (col, row) = self.vp.to_origin();
        let mut output: String = Default::default();

        output.push_str(&cursor::MoveTo(col - 1, row - 1).to_string());
        output.push_str(&style::SetBackgroundColor(te::BG_LAYER).to_string());
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
