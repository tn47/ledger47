use crossterm::{
    cursor,
    event::{
        self as ct_event, DisableMouseCapture, EnableMouseCapture, Event as CtEvent, KeyCode,
        KeyEvent, MouseEvent,
    },
    execute,
    style::{self, Color},
    terminal,
};
use jsondata::Json;
use unicode_width::UnicodeWidthStr;

use std::{
    ffi,
    io::{self, Write},
    marker,
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
        None => Application::<db_files::Db, Json>::new_workspace()?,
        Some(store) => todo!(),
    };

    app.event_loop()
}

pub struct View {
    pub(crate) tm: Terminal,
    pub(crate) layers: Vec<Layer>,
}

pub struct Application<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    view: View,
    store: Option<D>,

    _phantom_t: marker::PhantomData<T>,
}

impl<D, T> Application<D, T>
where
    D: Store<T>,
    T: ToString + FromStr,
{
    fn new_workspace() -> Result<Application<D, T>> {
        let mut app = Application {
            view: View {
                tm: Terminal::init()?,
                layers: Default::default(),
            },
            store: Default::default(),
            _phantom_t: marker::PhantomData,
        };
        let layer = tl::NewWorkspace::new_layer(&mut app.view)?;
        app.view.layers.push(layer);

        Ok(app)
    }

    fn event_loop(mut self) -> Result<()> {
        loop {
            let evnt: Event = err_at!(Fatal, ct_event::read())?.into();
            match evnt {
                Event::Resize { cols, rows } => {
                    self.resize(cols, rows)?.build()?.queue()?.flush();
                }
                Event::Key {
                    code: KeyCode::Char('q'),
                    modifiers,
                } if modifiers.is_empty() => break Ok(()),
                evnt => {
                    self.handle_event(evnt);
                }
            };
        }
    }

    fn resize(&mut self, _cols: u16, _rows: u16) -> Result<&mut Self> {
        self.view.tm = Terminal::init()?;
        let layers: Vec<Layer> = self.view.layers.drain(..).collect();
        for layer in layers.into_iter() {
            let layer = layer.resize(&mut self.view)?;
            self.view.layers.push(layer);
        }

        Ok(self)
    }

    fn build(&mut self) -> Result<&mut Self> {
        let mut layers: Vec<Layer> = self.view.layers.drain(..).collect();
        for layer in layers.iter_mut() {
            layer.build(&mut self.view)?;
        }
        self.view.layers = layers;

        Ok(self)
    }

    fn handle_event(&mut self, mut evnt: Event) -> Result<&mut Self> {
        let mut layers: Vec<Layer> = self.view.layers.drain(..).collect();
        for layer in layers.iter_mut().rev() {
            evnt = match layer.handle_event(&mut self.view, evnt)? {
                Some(evnt) => evnt,
                None => break,
            }
        }
        self.view.layers = layers;

        Ok(self)
    }

    #[inline]
    fn queue(&mut self) -> Result<&mut Self> {
        let mut layers: Vec<Layer> = self.view.layers.drain(..).collect();
        for layer in layers.iter() {
            err_at!(Fatal, execute!(self.view.tm.stdout, layer))?;
        }
        self.view.layers = layers;

        Ok(self)
    }

    #[inline]
    fn flush(&mut self) -> &mut Self {
        self.view.tm.stdout.flush();
        self
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
        terminal::enable_raw_mode();
        err_at!(Fatal, execute!(stdout, EnableMouseCapture, cursor::Hide))?;

        let (cols, rows) = err_at!(Fatal, terminal::size())?;
        Ok(Terminal { stdout, cols, rows })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        execute!(self.stdout, DisableMouseCapture, cursor::Show);
        terminal::disable_raw_mode();
    }
}
