use crossterm::Command;
use log::trace;
use unicode_width::UnicodeWidthStr;

use std::marker;

use crate::app::Application;
use crate::event::Event;
use crate::term_elements::{self as te};
use ledger::core::{Result, Store};

pub enum Layer<S>
where
    S: Store,
{
    NewWorkspace(NewWorkspace<S>),
}

impl<S> Layer<S>
where
    S: Store,
{
    pub fn refresh(&mut self, app: &mut Application<S>) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.refresh(app),
        }
    }

    pub fn handle_event(&mut self, app: &mut Application<S>, evnt: Event) -> Result<Option<Event>> {
        match self {
            Layer::NewWorkspace(layer) => layer.handle_event(app, evnt),
        }
    }
}

impl<S> Command for Layer<S>
where
    S: Store,
{
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        match self {
            Layer::NewWorkspace(layer) => layer.ansi_code(),
        }
    }
}

pub struct NewWorkspace<S>
where
    S: Store,
{
    vp: te::Viewport,
    elements: Vec<te::Element>,

    _phantom_s: marker::PhantomData<S>,
}

impl<S> NewWorkspace<S>
where
    S: Store,
{
    pub fn new(app: &mut Application<S>) -> Result<NewWorkspace<S>> {
        let vp = app.to_viewport();

        let border = te::Border::new(vp.clone()).ok().unwrap();
        let title = {
            let content = "Create new workspace".to_string();
            let title_vp = vp
                .clone()
                .move_by(2, 0)
                .resize_to(1, (content.width() as u16) + 2);
            te::Title::new(title_vp, &content).ok().unwrap()
        };
        let ws_input_name = {
            let prefix = "Enter workspace name : ";
            let input_vp = vp.clone().move_by(2, 3).resize_to(1, 60);
            te::EditLine::new(input_vp, prefix).ok().unwrap()
        };
        let comm_head = {
            let content = "Enter default commodity details";
            let comm_vp = vp.clone().move_by(2, 5).resize_to(1, 60);
            te::TextLine::new(comm_vp, content, te::FG_SECTION)
                .ok()
                .unwrap()
        };
        let comm_input_name = {
            let prefix = " Name : ";
            let comm_vp = vp.clone().move_by(5, 7).resize_to(1, 40);
            te::EditLine::new(comm_vp, prefix).ok().unwrap()
        };
        let comm_tags = {
            let prefix = " Tags : ";
            let comm_vp = vp.clone().move_by(5, 9).resize_to(1, 40);
            te::EditLine::new(comm_vp, prefix).ok().unwrap()
        };
        let comm_input_notes = {
            let prefix = "Notes : ";
            let comm_vp = vp.clone().move_by(5, 11).resize_to(10, 40);
            te::EditBox::new(comm_vp, prefix).ok().unwrap()
        };

        let elements = vec![
            te::Element::Border(border),
            te::Element::Title(title),
            te::Element::EditLine(ws_input_name),
            te::Element::TextLine(comm_head),
            te::Element::EditLine(comm_input_name),
            te::Element::EditLine(comm_tags),
            te::Element::EditBox(comm_input_notes),
        ];

        Ok(NewWorkspace {
            vp,
            elements,

            _phantom_s: marker::PhantomData,
        })
    }

    pub fn refresh(&mut self, app: &mut Application<S>) -> Result<()> {
        Ok(())
    }

    pub fn handle_event(
        &mut self,
        _app: &mut Application<S>,
        evnt: Event,
    ) -> Result<Option<Event>> {
        Ok(Some(evnt))
    }
}

impl<S> Command for NewWorkspace<S>
where
    S: Store,
{
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        let (col, row) = self.vp.to_origin();
        let (height, width) = self.vp.to_size();

        trace!(
            "NewWorkspace::Viewport col:{} row:{} height:{} width:{}",
            col,
            row,
            height,
            width
        );

        let mut output: String = Default::default();
        for element in self.elements.iter() {
            output.push_str(&element.to_string());
        }
        output
    }
}
