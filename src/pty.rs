use {
    cgmath::Vector2,
    r3vi::{
        view::{
            InnerViewPort,
            singleton::*
        },
        buffer::{
            singleton::*
        }
    },
    nested::{
        terminal::{TerminalEditorResult, TerminalEvent, TerminalView},
    },
    std::sync::{Arc, Mutex},
    termion::event::{Event, Key},
};

pub use portable_pty::CommandBuilder;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub enum PTYStatus {
    Running { pid: u32 },
    Done { status: portable_pty::ExitStatus },
}

impl Default for PTYStatus {
    fn default() -> Self {
        PTYStatus::Running { pid: 0 }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTY {
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
}

impl PTY {
    pub fn new(
        cmd: portable_pty::CommandBuilder,
        max_size: Vector2<i16>,
        term_port: InnerViewPort<dyn TerminalView>,
        status_port: InnerViewPort<dyn SingletonView<Item = PTYStatus>>,
    ) -> Option<Self> {
        // Create a new pty
        let pair = portable_pty::native_pty_system()
            .openpty(portable_pty::PtySize {
                rows: max_size.y as u16,
                cols: max_size.x as u16,

                // Not all systems support pixel_width, pixel_height,
                // but it is good practice to set it to something
                // that matches the size of the selected font.  That
                // is more complex than can be shown here in this
                // brief example though!
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        if let Ok(child) = pair.slave.spawn_command(cmd) {
            let mut reader = pair.master.try_clone_reader().unwrap();
            let mut status_buf = SingletonBuffer::with_port(
                PTYStatus::Running {
                    pid: child.process_id().expect(""),
                },
                status_port,
            );

            let child = Arc::new(Mutex::new(child));

            async_std::task::spawn_blocking(move || {
                nested::terminal::ansi_parser::read_ansi_from(&mut reader, max_size, term_port);
            });

            async_std::task::spawn_blocking({
                let child = child.clone();
                move || loop {
                    if let Ok(Some(status)) = child.lock().unwrap().try_wait() {
                        status_buf.set(PTYStatus::Done { status });
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });

            Some(PTY {
                master: Mutex::new(pair.master),
                child,
            })
        } else {
            None
        }
    }

    pub fn kill(&mut self) {
        self.child.lock().unwrap().kill().unwrap();
    }

    pub fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.master.lock().unwrap().write(&[13]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                write!(self.master.lock().unwrap(), "{}", c).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Esc)) => {
                self.master.lock().unwrap().write(&[0x1b]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                self.master.lock().unwrap().write(&[0x8]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::F(n))) => {
                self.master
                    .lock()
                    .unwrap()
                    .write(&[
                        0x1b,
                        0x0a,
                        match n {
                            11 => 133,
                            12 => 134,
                            n => 58 + n,
                        },
                    ])
                    .unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Up)) => {
                self.master
                    .lock()
                    .unwrap()
                    .write(&[b'\x1B', b'[', b'A'])
                    .unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Down)) => {
                self.master
                    .lock()
                    .unwrap()
                    .write(&[b'\x1B', b'[', b'B'])
                    .unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Right)) => {
                self.master
                    .lock()
                    .unwrap()
                    .write(&[b'\x1B', b'[', b'C'])
                    .unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Left)) => {
                self.master
                    .lock()
                    .unwrap()
                    .write(&[b'\x1B', b'[', b'D'])
                    .unwrap();
                TerminalEditorResult::Continue
            }
            _ => TerminalEditorResult::Exit,
        }
    }
}
