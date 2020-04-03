use crossterm::{
    cursor,
    event::{self as ct_event, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute, queue,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use jsondata::Json;
use log::{debug, info, trace};

use std::{
    ffi,
    io::{self, Write},
    str::FromStr,
};

use crate::event::Event;
use crate::term_elements as te;
use crate::term_layers::{self as tl, Layer};
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
        None => Application::<db_files::Db, Json>::new_workspace(dir.to_os_string())?,
        Some(_store) => todo!(),
    };

    app.event_loop()
}

enum ViewFocus {
    Layer,
    Cmd,
}

pub struct View<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    tm: Terminal,
    vp: te::Viewport,
    layers: Vec<Layer<D, T>>,
    status: te::StatusLine,
    // cmd: te::CmdLine,
    focus: ViewFocus,
}

impl<D, T> View<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    pub fn new() -> Result<View<D, T>> {
        let tm = err_at!(Fatal, Terminal::init())?;
        let vp = te::Viewport::new(1, 1, tm.rows - 1, tm.cols);
        let status = {
            let vp = vp.clone().move_to(1, vp.to_bottom()).resize_to(1, tm.cols);
            te::StatusLine::new(te::Coordinates::new(vp))?
        };

        debug!("App view-port {}", vp);

        Ok(View {
            tm,
            vp,
            layers: Default::default(),
            status,
            focus: ViewFocus::Layer,
        })
    }

    #[inline]
    pub fn to_viewport(&self) -> te::Viewport {
        self.vp.clone()
    }
}

pub struct Application<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    dir: ffi::OsString,
    view: View<D, T>,
    store: Option<D>,
}

impl<D, T> Application<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    fn new_workspace(dir: ffi::OsString) -> Result<Application<D, T>> {
        let mut app = Application {
            dir: dir.clone(),
            view: View::new()?,
            store: Default::default(),
        };
        let layer = tl::NewWorkspace::new_layer(&mut app)?;
        app.view.layers.push(layer);

        info!("New workspace dir:{:?}", dir);

        Ok(app)
    }

    fn new(dir: ffi::OsString) -> Result<Application<D, T>> {
        let mut app = Application {
            dir: dir.clone(),
            view: View::new()?,
            store: Default::default(),
        };
        // TODO: change this to different view.
        let layer = tl::NewWorkspace::new_layer(&mut app)?;
        app.view.layers.push(layer);

        info!("Open workspace dir:{:?}", dir);

        Ok(app)
    }

    fn event_loop(mut self) -> Result<()> {
        self.build()?.queue()?.flush()?;
        self.status_log("");

        loop {
            let evnt: Event = err_at!(Fatal, ct_event::read())?.into();
            self.status_log("");

            trace!("Event-{}", evnt);

            match evnt {
                Event::Resize { cols, rows } => {
                    self.resize(cols, rows)?.build()?.queue()?.flush()?;
                }
                evnt => match self.handle_event(evnt)? {
                    Some(Event::Key {
                        code: KeyCode::Char('q'),
                        modifiers,
                    }) if modifiers.is_empty() => break Ok(()),
                    _ => (),
                },
            };
        }
    }

    fn handle_event(&mut self, mut evnt: Event) -> Result<Option<Event>> {
        let mut layers: Vec<Layer<D, T>> = self.view.layers.drain(..).collect();
        let mut iter = layers.iter_mut().rev();
        loop {
            if let Some(layer) = iter.next() {
                evnt = match layer.handle_event(self, evnt)? {
                    Some(evnt) => evnt,
                    None => break Ok(None),
                }
            } else {
                self.view.layers = layers;
                break Ok(Some(evnt));
            }
        }
    }

    fn resize(&mut self, _cols: u16, _rows: u16) -> Result<&mut Self> {
        self.view.tm = Terminal::init()?;
        let layers: Vec<Layer<D, T>> = self.view.layers.drain(..).collect();
        for layer in layers.into_iter() {
            let layer = layer.resize(self)?;
            self.view.layers.push(layer);
        }

        Ok(self)
    }

    fn build(&mut self) -> Result<&mut Self> {
        let mut layers: Vec<Layer<D, T>> = self.view.layers.drain(..).collect();
        for layer in layers.iter_mut() {
            layer.build(self)?;
        }
        self.view.layers = layers;

        Ok(self)
    }

    fn queue(&mut self) -> Result<&mut Self> {
        let layers: Vec<Layer<D, T>> = self.view.layers.drain(..).collect();
        for layer in layers.iter() {
            err_at!(Fatal, queue!(self.view.tm.stdout, layer))?;
        }
        self.view.layers = layers;

        Ok(self)
    }

    pub fn status_log(&mut self, msg: &str) -> Result<()> {
        self.view.status.log(msg);
        err_at!(Fatal, execute!(self.view.tm.stdout, self.view.status))?;
        if msg.len() > 0 {
            debug!("Status <- {}", msg);
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<&mut Self> {
        err_at!(Fatal, self.view.tm.stdout.flush())?;
        Ok(self)
    }

    #[inline]
    pub fn to_viewport(&self) -> te::Viewport {
        self.view.to_viewport()
    }

    #[inline]
    pub fn as_mut_status(&mut self) -> &mut te::StatusLine {
        &mut self.view.status
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
