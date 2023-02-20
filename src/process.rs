use {
    crate::pty::{PTYStatus, PTY},
    r3vi::{
        view::{
            OuterViewPort, ViewPort,
            singleton::*,
            sequence::*,
        }
    },
    nested::{
        editors::{
            list::{ListCursorMode},  
        },
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEvent, TerminalStyle,
            TerminalView,
            widgets::ascii_box::AsciiBox
        },
        tree::{TreeCursor, TreeNav, NestedNode}
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProcessLauncher {
    cmd_editor: NestedNode,
    pty: Option<crate::pty::PTY>,
    _ptybox: Arc<RwLock<AsciiBox>>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,
    status_port: ViewPort<dyn SingletonView<Item = PTYStatus>>,

    comp_port: ViewPort<dyn TerminalView>,
    _compositor: Arc<RwLock<nested::terminal::TerminalCompositor>>,
}

impl ProcessLauncher {
    pub fn new(
        cmd_editor: NestedNode
    ) -> Self {
        let pty_port = ViewPort::new();
        let status_port = ViewPort::new();
        let comp_port = ViewPort::new();
        let box_port = ViewPort::<dyn TerminalView>::new();
        let compositor = nested::terminal::TerminalCompositor::new(comp_port.inner());

        compositor.write().unwrap().push(
            box_port
                .outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((90, 120, 100)))),
        );
        compositor.write().unwrap().push(
            cmd_editor.view.clone().unwrap()
        );

        ProcessLauncher {
            cmd_editor,
            pty: None,
            _ptybox: AsciiBox::new(
                cgmath::Vector2::new(0, 0),
                pty_port.outer().map_item(|_, a: &TerminalAtom| {
                    a.add_style_back(TerminalStyle::fg_color((230, 230, 230)))
                }),
                box_port.inner(),
            ),
            suspended: false,
            pty_port,
            status_port,
            comp_port,
            _compositor: compositor,
        }
    }

    pub fn launch_pty(&mut self) {
        let ctx = self.cmd_editor.ctx.clone().unwrap();

        let mut strings = Vec::<String>::new();

        if let Some(data) = self.cmd_editor.data.clone() {
            let v = data.read().unwrap()
                .descend((&ctx, "( List ProcessArg )"))
                .unwrap()
                .read().unwrap()
                .get_view::<dyn SequenceView<Item = NestedNode>>();

            for i in 0..v.len().unwrap_or(0) {
                if let Some(arg_data) = v
                    .get(&i)
                    .unwrap()
                    .data
                    .clone()
                {
                    let arg_view = arg_data
                        .read()
                        .unwrap()
                        .descend((&ctx, "( List Char )"))
                        .unwrap()
                        .read().unwrap()
                        .get_view::<dyn SequenceView<Item = NestedNode>>()
                        .unwrap();

                    let arg = arg_view.iter().filter_map(
                            |node| {
                                if let Some(c_data) = node.data.clone() {
                                    if let Some(c_view) = c_data
                                        .read().unwrap()
                                        .get_view::<dyn SingletonView<Item = Option<char>>>()
                                    {
                                        c_view.get()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                    ).collect::<String>();

                    strings.push(arg);
                }
            }            
        }

        if strings.len() > 0 {
            // Spawn a shell into the pty
            let mut cmd = crate::pty::CommandBuilder::new(strings[0].as_str());
            cmd.args(&strings[1..]);
            cmd.cwd(".");

            self.cmd_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![],
            });

            self.pty = PTY::new(
                cmd,
                cgmath::Vector2::new(120, 40),
                self.pty_port.inner(),
                self.status_port.inner(),
            );
        }
    }

    pub fn is_captured(&self) -> bool {
        self.pty.is_some() && !self.suspended
    }

    pub fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.comp_port.outer()
    }
}


use nested::type_system::ReprTree;
use nested::commander::ObjCommander;

impl ObjCommander for ProcessLauncher {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {

        // todo: move to observer of status view
        if let PTYStatus::Done { status: _ } = self.status_port.outer().get_view().get() {
            self.pty = None;
            self.suspended = false;
        }


        let ctx = self.cmd_editor.ctx.clone().unwrap();
        let ctx = ctx.read().unwrap();

        let co = cmd_obj.read().unwrap();
        let cmd_type = co.get_type().clone();
        let term_event_type = ctx.type_term_from_str("( TerminalEvent )").unwrap();
        let char_type = ctx.type_term_from_str("( Char )").unwrap();

        if cmd_type == term_event_type {
            if let Some(te_view) = co.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                drop(co);
                let event = te_view.get();
                
                match event {
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => {
                        // todo: sigterm instead of kill?
                        if let Some(pty) = self.pty.as_mut() {
                            pty.kill();
                        }

                        self.pty = None;
                        self.suspended = false;
                        self.cmd_editor.goto(TreeCursor {
                            leaf_mode: ListCursorMode::Insert,
                            tree_addr: vec![],
                        });
                    }
                    TerminalEvent::Input(Event::Key(Key::Ctrl('z'))) => {
                        self.suspended = true;
                        self.cmd_editor.goto(TreeCursor {
                            leaf_mode: ListCursorMode::Insert,
                            tree_addr: vec![],
                        });
                    }
                    event => {
                        if let Some(pty) = self.pty.as_mut() {
                            pty.handle_terminal_event(&event);
                        } else {
                            match event {
                                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                                    // launch command
                                    self.launch_pty();
                                }
                                _event => self.cmd_editor.send_cmd_obj(cmd_obj),
                            }
                        }
                    }
                }
            }
        } else if cmd_type == char_type {

            if let Some(cmd_view) = co.get_view::<dyn SingletonView<Item = char>>() {
                drop(co);
                let c = cmd_view.get();
                
                if c == '\n' {
                    self.launch_pty();
                } else {
                    self.cmd_editor.send_cmd_obj(cmd_obj);
                }
                
            } else {
                drop(co);
                self.cmd_editor.send_cmd_obj(cmd_obj);
            }
        } else {
            drop(co);
            self.cmd_editor.send_cmd_obj(cmd_obj);
        }
    }
}

