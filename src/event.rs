use crossterm::event::{self, KeyCode, KeyModifiers, MouseButton};

pub enum Event {
    Resize {
        cols: u16,
        rows: u16,
    },
    Key {
        code: KeyCode,
        modifiers: KeyModifiers,
    },
    MouseDown {
        button: MouseButton,
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
    },
    MouseUp {
        button: MouseButton,
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
    },
    MouseDrag {
        button: MouseButton,
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
    },
    MouseScrollDown {
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
    },
    MouseScrollUp {
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
    },
}

impl From<event::Event> for Event {
    fn from(e: event::Event) -> Event {
        use event::MouseEvent::{Down, Drag, ScrollDown, ScrollUp, Up};

        match e {
            event::Event::Resize(cols, rows) => Event::Resize { cols, rows },
            event::Event::Key(event::KeyEvent { code, modifiers }) => {
                Event::Key { code, modifiers }
            }
            event::Event::Mouse(m) => match m {
                Down(button, col, row, modifiers) => Event::MouseDown {
                    button,
                    col,
                    row,
                    modifiers,
                },
                Up(button, col, row, modifiers) => Event::MouseUp {
                    button,
                    col,
                    row,
                    modifiers,
                },
                Drag(button, col, row, modifiers) => Event::MouseDrag {
                    button,
                    col,
                    row,
                    modifiers,
                },
                ScrollDown(col, row, modifiers) => Event::MouseScrollDown {
                    col,
                    row,
                    modifiers,
                },
                ScrollUp(col, row, modifiers) => Event::MouseScrollUp {
                    col,
                    row,
                    modifiers,
                },
            },
        }
    }
}
