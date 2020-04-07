use crossterm::{
    cursor,
    event::{Event, KeyCode},
    style, Command as TermCommand,
};
use log::trace;
use unicode_width::UnicodeWidthStr;

use std::{iter::FromIterator, marker};

use crate::app::Application;
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
    pub fn focus(&mut self, app: &mut Application<S>) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.focus(app),
        }
    }

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

    pub fn leave(&mut self, app: &mut Application<S>) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.leave(app),
        }
    }
}

impl<S> TermCommand for Layer<S>
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
    refresh: usize,
    in_focus: bool,
    focus: Vec<usize>,

    _phantom_s: marker::PhantomData<S>,
}

impl<S> NewWorkspace<S>
where
    S: Store,
{
    const DEFAULT_FOCUS: usize = 2;

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
            let inline = "Enter workspace name, only alphanumeric and '_'";
            let input_vp = vp.clone().move_by(5, 3).resize_to(1, 40);
            te::EditLine::new(input_vp, inline).ok().unwrap()
        };
        let comm_head = {
            let content = "Enter default commodity details";
            let comm_vp = vp.clone().move_by(5, 5).resize_to(1, 60);
            te::TextLine::new(comm_vp, content, te::FG_SECTION)
                .ok()
                .unwrap()
        };
        let comm_input_name = {
            let inline = "Name of the commodity, only alphanumeric";
            let comm_vp = vp.clone().move_by(8, 7).resize_to(1, 60);
            te::EditLine::new(comm_vp, inline).ok().unwrap()
        };
        let comm_tags = {
            let inline = "List of tags, EG: money.asia,exchange.westernunion";
            let comm_vp = vp.clone().move_by(8, 9).resize_to(1, 60);
            te::EditLine::new(comm_vp, inline).ok().unwrap()
        };
        let comm_input_notes = {
            let inline = "Any notes for user consumption";
            let comm_vp = vp.clone().move_by(8, 11).resize_to(10, 60);
            te::EditBox::new(comm_vp, inline).ok().unwrap()
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
            refresh: Default::default(),
            in_focus: Default::default(),
            focus: vec![2, 4, 5, 6],

            _phantom_s: marker::PhantomData,
        })
    }

    pub fn refresh(&mut self, _app: &mut Application<S>) -> Result<()> {
        self.refresh += 1;
        Ok(())
    }

    pub fn focus(&mut self, app: &mut Application<S>) -> Result<()> {
        self.focus_element(app)?;
        self.in_focus = true;
        Ok(())
    }

    pub fn leave(&mut self, app: &mut Application<S>) -> Result<()> {
        let off = self.focus.first().unwrap().clone();
        self.elements[off].leave(app)?;

        self.in_focus = Default::default();
        self.refresh = Default::default();
        Ok(())
    }

    pub fn handle_event(&mut self, app: &mut Application<S>, evnt: Event) -> Result<Option<Event>> {
        let off = self.focus.first().unwrap().clone();
        self.elements[off].handle_event(app, evnt)?;

        match (te::to_modifiers(&evnt), te::to_key_code(&evnt)) {
            (modifiers, Some(code)) if modifiers.is_empty() => match code {
                KeyCode::Enter | KeyCode::Tab => {
                    let off = self.focus.remove(0);
                    self.focus.push(off);
                    self.focus_element(app)?;
                    Ok(None)
                }
                KeyCode::BackTab => {
                    let off = self.focus.pop().unwrap();
                    self.focus.insert(0, off);
                    self.focus_element(app)?;
                    Ok(None)
                }
                _ => Ok(Some(evnt)),
            },
            _ => Ok(Some(evnt)),
        }
    }

    fn focus_element(&mut self, app: &mut Application<S>) -> Result<()> {
        let off = self.focus.first().unwrap().clone();
        match &mut self.elements[off] {
            te::Element::EditLine(e) => e.clear_inline()?,
            te::Element::EditBox(e) => e.clear_inline()?,
            _ => (),
        };

        trace!("Focus layer_new_workspace off:{}", off);

        self.elements[off].focus(app)?;

        Ok(())
    }
}

impl<S> TermCommand for NewWorkspace<S>
where
    S: Store,
{
    type AnsiType = String;

    fn ansi_code(&self) -> Self::AnsiType {
        use std::iter::repeat;

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
        if self.refresh < 2 {
            let s = String::from_iter(repeat(' ').take(width as usize));
            for r in 0..height {
                output.push_str(&cursor::MoveTo(col - 1, row + r).to_string());
                output.push_str(&style::style(&s).on(te::BG_LAYER).to_string());
            }
        }
        for element in self.elements.iter() {
            output.push_str(&element.to_string());
        }

        output
    }
}
