use chrono;
use crossterm::{
    cursor,
    event::{self as ct_event, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, queue,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info, trace};

use std::{
    ffi,
    io::{self, Write},
    mem,
};

use crate::term_elements as te;
use crate::term_layers::{self as tl, Layer};
use crate::util;
use crate::Opt;

use ledger::{
    core::{Error, Result, Store},
    db_files,
};

pub fn run(opts: Opt) -> Result<()> {
    let dir: &ffi::OsStr = opts.dir.as_ref();
    let store = match db_files::Db::open(dir) {
        Ok(store) => Ok(Some(store)),
        Err(Error::NotFound(_)) => Ok(None),
        Err(err) => Err(err),
    }?;

    let app = match store {
        None => Application::<db_files::Db>::new_workspace(dir.to_os_string())?,
        Some(_store) => todo!(),
    };

    app.event_loop()
}

enum ViewFocus {
    Layer,
    Cmd,
}

pub struct View<S>
where
    S: Store,
{
    tm: Terminal,
    vp: te::Viewport,
    head: te::HeadLine,
    layers: Vec<Layer<S>>,
    status: te::StatusLine,
    // cmd: te::CmdLine,
    focus: ViewFocus,
    cursor: Option<(u16, u16)>,
}

impl<S> View<S>
where
    S: Store,
{
    pub fn new() -> Result<View<S>> {
        let tm = err_at!(Fatal, Terminal::init())?;
        // adjust full screen for a head-line in top and status-line at bottom.
        let vp = te::Viewport::new(1, 2, tm.rows - 2, tm.cols);

        debug!("App view-port {}", vp);

        Ok(View {
            tm,
            vp,
            head: Default::default(),
            layers: Default::default(),
            status: Default::default(),
            focus: ViewFocus::Layer,
            cursor: Some((1, 1)),
        })
    }

    #[inline]
    pub fn to_viewport(&self) -> te::Viewport {
        self.vp.clone()
    }
}

pub struct Application<S>
where
    S: Store,
{
    dir: ffi::OsString,
    view: View<S>,
    store: Option<S>,
    date: chrono::Date<chrono::Local>,
    period: (chrono::Date<chrono::Local>, chrono::Date<chrono::Local>),
}

impl<S> Application<S>
where
    S: Store,
{
    fn new_workspace(dir: ffi::OsString) -> Result<Application<S>> {
        let mut app = Application {
            dir: dir.clone(),
            view: View::new()?,
            store: Default::default(),
            date: chrono::Local::now().date(),
            period: util::date_to_period(chrono::Local::now().date()),
        };

        app.view.head = {
            let vp = te::Viewport::new(1, 1, 1, app.view.tm.cols);
            te::HeadLine::new(vp, &mut app)?
        };
        app.view.status = {
            let vp = te::Viewport::new(1, app.view.tm.rows, 1, app.view.tm.cols);
            te::StatusLine::new(vp)?
        };

        let layer = tl::NewWorkspace::new(&mut app)?;
        app.view.layers.push(Layer::NewWorkspace(layer));

        info!("New workspace dir:{:?}", dir);

        Ok(app)
    }

    fn new(dir: ffi::OsString) -> Result<Application<S>> {
        let mut app = Application {
            dir: dir.clone(),
            view: View::new()?,
            store: Default::default(),
            date: chrono::Local::now().date(),
            period: util::date_to_period(chrono::Local::now().date()),
        };

        app.view.head = {
            let vp = te::Viewport::new(1, 1, 1, app.view.tm.cols);
            te::HeadLine::new(vp, &mut app)?
        };
        app.view.status = {
            let vp = te::Viewport::new(1, app.view.tm.rows, 1, app.view.tm.cols);
            te::StatusLine::new(vp)?
        };

        let layer = tl::NewWorkspace::new(&mut app)?;
        app.view.layers.push(Layer::NewWorkspace(layer));

        info!("Open workspace dir:{:?}", dir);

        Ok(app)
    }

    fn event_loop(mut self) -> Result<()> {
        self.view.status.log("");
        self.refresh()?.render()?;

        match self.view.layers.pop() {
            Some(mut layer) => {
                layer.focus(&mut self)?;
                self.view.layers.push(layer);
            }
            None => (),
        }

        loop {
            self.show_cursor()?;
            let evnt = err_at!(Fatal, ct_event::read())?;

            trace!("Event-{:?}", evnt);

            let evnt = match evnt {
                Event::Resize { .. } => None,
                evnt => self.handle_event(evnt)?,
            };

            if let Some(evnt) = evnt {
                match (te::to_modifiers(&evnt), te::to_key_code(&evnt)) {
                    (m, Some(KeyCode::Char('q'))) if m.is_empty() => break Ok(()),
                    (m, Some(KeyCode::Esc)) if m.is_empty() => {
                        if self.view.layers.len() > 1 {
                            self.view.layers.pop();
                        }
                    }
                    _ => (),
                };
            };

            self.refresh()?.render()?;
        }
    }

    fn handle_event(&mut self, mut evnt: Event) -> Result<Option<Event>> {
        let mut layers: Vec<Layer<S>> = self.view.layers.drain(..).collect();
        let mut iter = layers.iter_mut().rev();
        let evnt = loop {
            if let Some(layer) = iter.next() {
                evnt = match layer.handle_event(self, evnt)? {
                    Some(evnt) => evnt,
                    None => break None,
                }
            } else {
                break Some(evnt);
            }
        };
        self.view.layers = layers;

        Ok(evnt)
    }

    fn refresh(&mut self) -> Result<&mut Self> {
        let mut head = mem::replace(&mut self.view.head, Default::default());
        head.refresh(self)?;
        self.view.head = head;

        let mut layers: Vec<Layer<S>> = self.view.layers.drain(..).collect();
        for layer in layers.iter_mut() {
            layer.refresh(self)?;
        }
        self.view.layers = layers;

        let mut status = mem::replace(&mut self.view.status, Default::default());
        status.refresh(self)?;
        self.view.status = status;

        Ok(self)
    }

    fn render(&mut self) -> Result<&mut Self> {
        err_at!(Fatal, queue!(self.view.tm.stdout, self.view.head))?;

        match self.view.layers.pop() {
            Some(layer) => {
                err_at!(Fatal, queue!(self.view.tm.stdout, layer))?;
                self.view.layers.push(layer);
            }
            None => (),
        }

        err_at!(Fatal, queue!(self.view.tm.stdout, self.view.status))?;

        err_at!(Fatal, self.view.tm.stdout.flush())?;

        Ok(self)
    }
}

impl<S> Application<S>
where
    S: Store,
{
    #[inline]
    pub fn to_viewport(&self) -> te::Viewport {
        self.view.to_viewport()
    }

    #[inline]
    pub fn to_local_date(&self) -> chrono::Date<chrono::Local> {
        self.date.clone()
    }

    #[inline]
    pub fn to_local_period(&self) -> (chrono::Date<chrono::Local>, chrono::Date<chrono::Local>) {
        self.period.clone()
    }

    #[inline]
    pub fn as_mut_status(&mut self) -> &mut te::StatusLine {
        &mut self.view.status
    }

    #[inline]
    pub fn set_date(&mut self, date: chrono::Date<chrono::Local>) -> &mut Self {
        self.date = date;
        self.period = util::date_to_period(date);
        self
    }

    #[inline]
    pub fn send_status(&mut self, msg: &str) -> &mut Self {
        self.view.status.log(msg);
        self
    }

    #[inline]
    pub fn set_period(
        &mut self,
        period: (chrono::Date<chrono::Local>, chrono::Date<chrono::Local>),
    ) -> &mut Self {
        self.period = period;
        self
    }

    pub fn show_cursor(&mut self) -> Result<()> {
        match self.view.cursor {
            Some((col, row)) => err_at!(
                Fatal,
                execute!(
                    self.view.tm.stdout,
                    cursor::MoveTo(col - 1, row - 1),
                    cursor::EnableBlinking,
                    cursor::Show,
                )
            )?,
            None => err_at!(Fatal, execute!(self.view.tm.stdout, cursor::Hide,))?,
        }

        Ok(())
    }

    pub fn hide_cursor(&mut self) -> Result<()> {
        trace!("move cursor {:?}->None", cursor::position());
        self.view.cursor = None;

        Ok(())
    }

    pub fn move_cursor(&mut self, col: u16, row: u16) -> Result<()> {
        trace!("move cursor {:?}->{:?}", cursor::position(), (col, row));
        self.view.cursor = Some((col, row));

        Ok(())
    }
}

pub struct Terminal {
    pub(crate) stdout: io::Stdout,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
}

impl Terminal {
    fn init() -> Result<Terminal> {
        let mut stdout = io::stdout();
        err_at!(Fatal, terminal::enable_raw_mode())?;
        err_at!(
            Fatal,
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableMouseCapture,
                cursor::Hide
            )
        )?;

        let (cols, rows) = err_at!(Fatal, terminal::size())?;
        Ok(Terminal { stdout, cols, rows })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        execute!(
            self.stdout,
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}
