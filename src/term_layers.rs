use crossterm::{cursor, event::KeyCode, style, Command as TermCommand};
use log::trace;

use std::{iter::FromIterator, marker};

use crate::{
    app::Application,
    event::Event,
    term_elements::{self as te},
};
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

    pub fn refresh(&mut self, app: &mut Application<S>, force: bool) -> Result<()> {
        match self {
            Layer::NewWorkspace(layer) => layer.refresh(app, force),
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
    focus: TabOffsets,

    _phantom_s: marker::PhantomData<S>,
}

impl<S> NewWorkspace<S>
where
    S: Store,
{
    pub fn new(app: &mut Application<S>) -> Result<NewWorkspace<S>> {
        let vp = app.to_viewport();

        let border = te::Border::new(app, vp.clone(), "Create new workspace".to_string())
            .ok()
            .unwrap();
        let ws_input_name = {
            let input_vp = vp.clone().move_by(5, 3).resize_to(1, 60);
            let mut em = te::EditLine::new(app, input_vp).ok().unwrap();
            em.set_inline("Enter workspace name, only alphanumeric and '_'")
                .set_mandatory(true);
            em.refresh(app, true /*force*/)?;
            em
        };
        let comm_head = {
            let content = "Enter default commodity details";
            let comm_vp = vp.clone().move_by(5, 5).resize_to(1, 60);
            let mut em = te::Span::new(app, comm_vp, content).ok().unwrap();
            em.set_fg_color(te::FG_SECTION);
            em
        };
        let comm_input_name = {
            let comm_vp = vp.clone().move_by(8, 7).resize_to(1, 60);
            let mut em = te::EditLine::new(app, comm_vp).ok().unwrap();
            em.set_inline("Name of the commodity, only alphanumeric")
                .set_mandatory(true)
                .set_field("Name    :");
            em
        };
        let comm_input_symbol = {
            let comm_vp = vp.clone().move_by(8, 9).resize_to(1, 60);
            let mut em = te::EditLine::new(app, comm_vp).ok().unwrap();
            em.set_inline("Symbol for commodity, EG: '₹'")
                .set_field("Symbol  :");
            em
        };
        let comm_input_aliases = {
            let comm_vp = vp.clone().move_by(8, 11).resize_to(1, 60);
            let mut em = te::EditLine::new(app, comm_vp).ok().unwrap();
            em.set_inline("Comman separated list of aliases")
                .set_field("Aliases :");
            em
        };
        let comm_tags = {
            let comm_vp = vp.clone().move_by(8, 13).resize_to(1, 60);
            let mut em = te::EditLine::new(app, comm_vp).ok().unwrap();
            em.set_inline("List of tags, EG: money.asia,exchange.westernunion")
                .set_field("Tags    :");
            em
        };
        let comm_input_notes = {
            let comm_vp = vp.clone().move_by(8, 15).resize_to(7, 60);
            let mut em = te::EditBox::new(app, comm_vp).ok().unwrap();
            em.set_inline("Any notes for user consumption")
                .set_field("Notes   :");
            em
        };
        let button_ok = {
            let button_vp = vp.clone().move_by(18, 23).resize_to(1, 4);
            let mut em = te::Button::new(app, button_vp, "ok", te::ButtonType::Submit)
                .ok()
                .unwrap();
            em.set_bold(true);
            em
        };

        let elements = vec![
            te::Element::Border(border),
            te::Element::EditLine(ws_input_name),
            te::Element::Span(comm_head),
            te::Element::EditLine(comm_input_name),
            te::Element::EditLine(comm_input_symbol),
            te::Element::EditLine(comm_input_aliases),
            te::Element::EditLine(comm_tags),
            te::Element::EditBox(comm_input_notes),
            te::Element::Button(button_ok),
        ];

        Ok(NewWorkspace {
            vp,
            elements,
            focus: TabOffsets::new(vec![1, 3, 4, 5, 6, 7, 8, 0]),

            _phantom_s: marker::PhantomData,
        })
    }
}

impl<S> NewWorkspace<S>
where
    S: Store,
{
    pub fn refresh(&mut self, app: &mut Application<S>, force: bool) -> Result<()> {
        for em in self.elements.iter_mut() {
            em.refresh(app, force)?
        }
        Ok(())
    }

    pub fn focus(&mut self, app: &mut Application<S>) -> Result<()> {
        self.focus_element(app)?;
        Ok(())
    }

    pub fn leave(&mut self, app: &mut Application<S>) -> Result<()> {
        let off = self.focus.current();
        self.elements[off as usize].leave(app)?;
        Ok(())
    }

    pub fn handle_event(&mut self, app: &mut Application<S>, evnt: Event) -> Result<Option<Event>> {
        let off = self.focus.current();
        let evnt = self.elements[off as usize].handle_event(app, evnt)?;

        match evnt {
            Some(Event::Submit) => Ok(None),
            Some(evnt) => match (evnt.to_modifiers(), evnt.to_key_code()) {
                (m, Some(code)) => match code {
                    KeyCode::Esc if m.is_empty() => match self.focus.tab_to(0) {
                        Some(old_off) => {
                            self.elements[old_off].leave(app)?;
                            self.focus_element(app)?;
                            app.hide_cursor()?;
                            Ok(None)
                        }
                        None => Ok(None),
                    },
                    KeyCode::Enter | KeyCode::Tab => {
                        let old_off = self.focus.tab();
                        self.elements[old_off].leave(app)?;
                        self.focus_element(app)?;
                        Ok(None)
                    }
                    KeyCode::BackTab => {
                        let old_off = self.focus.back_tab();
                        self.elements[old_off].leave(app)?;
                        self.focus_element(app)?;
                        Ok(None)
                    }
                    _ => Ok(Some(evnt)),
                },
                _ => Ok(Some(evnt)),
            },
            None => Ok(None),
        }
    }

    fn focus_element(&mut self, app: &mut Application<S>) -> Result<()> {
        let em_idx = self.focus.current();
        trace!("Focus layer_new_workspace em_idx:{}", em_idx);

        if em_idx == 0 {
            self.elements[em_idx].focus(app)?;
            app.hide_cursor()?;
        } else {
            self.elements[em_idx].focus(app)?;
        }

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
        let s = String::from_iter(repeat(' ').take(width as usize));
        for r in 0..height {
            output.push_str(&cursor::MoveTo(col - 1, row + r).to_string());
            output.push_str(&style::style(&s).on(te::BG_LAYER).to_string());
        }
        for element in self.elements.iter() {
            output.push_str(&element.to_string());
        }

        output
    }
}

struct TabOffsets(Vec<usize>);

impl TabOffsets {
    fn new(offs: Vec<usize>) -> TabOffsets {
        TabOffsets(offs)
    }

    fn current(&self) -> usize {
        self.0.first().unwrap().clone()
    }

    fn tab(&mut self) -> usize {
        let old_off = self.0.remove(0);
        self.0.push(old_off);
        old_off
    }

    fn back_tab(&mut self) -> usize {
        let old_off = self.current();
        let off = self.0.pop().unwrap();
        self.0.insert(0, off);
        old_off
    }

    fn tab_to(&mut self, off: usize) -> Option<usize> {
        for (i, val) in self.0.clone().into_iter().enumerate() {
            if off == val && i == 0 {
                return None;
            } else if off == val {
                let old_off = self.0.remove(i);
                self.0.insert(0, off);
                return Some(old_off);
            }
        }
        unreachable!()
    }
}
