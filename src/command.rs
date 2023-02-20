
use {
    r3vi::{
        view::{
            AnyOuterViewPort, OuterViewPort, ViewPort,
            singleton::*,
            sequence::*,
        },
        buffer::index_hashmap::*
    },
    nested::{
        editors::{list::{ListEditor, ListCursorMode}, sum::SumEditor},
        terminal::{
            TerminalAtom, TerminalStyle, TerminalView,
            widgets::ascii_box::AsciiBox, TerminalEvent
        },
        tree::{NestedNode, TreeNav, TreeCursor},
        type_system::{Context, ReprTree},
        commander::ObjCommander,
        PtySegment
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::io::{Read, Write},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},

    crate::pipeline::PipelineLauncher
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
enum CommandState {
    Incubator(Arc<RwLock<PipelineLauncher>>),
    CD(NestedNode),
    Pipeline(Arc<RwLock<PipelineLauncher>>),
}

pub struct Command {
    ctx: Arc<RwLock<Context>>,
    grid: r3vi::buffer::index_hashmap::IndexBuffer<Point2<i16>, OuterViewPort<dyn TerminalView>>,

    comp_port: ViewPort<dyn TerminalView>,
    cwd_node: NestedNode,
    state: CommandState,

    cwd: String,

    sum_editor: NestedNode
}

impl Into<NestedNode> for Command {
    fn into(self) -> NestedNode {
        self.into_node()
    }
}

impl Command {
    pub fn into_node(self) -> NestedNode {
        self.sum_editor.clone()
            .set_view(self.comp_port.outer())
            .set_cmd(Arc::new(RwLock::new(self)))
    }

    pub fn new(ctx: Arc<RwLock<Context>>, cwd: String) -> Self {
        let mut cwd_node = Context::make_node(&ctx, (&ctx, "( Path )").into(), 0).unwrap();

        cwd_node.goto(TreeCursor::home());
        for c in cwd.chars() {
            let buf = r3vi::buffer::singleton::SingletonBuffer::new(c);

            cwd_node.send_cmd_obj(
                ReprTree::new_leaf((&ctx, "( Char )"), AnyOuterViewPort::from(buf.get_port()))
            );
        }
        cwd_node.goto(TreeCursor::none());

        let mut grid = IndexBuffer::new();
        let mut incubator_node = Context::make_node(&ctx, (&ctx, "( Pipeline )").into(), 1).unwrap();
        let mut path_node = Context::make_node(&ctx, (&ctx, "( Path )").into(), 2).unwrap();

        let mut sum_editor = SumEditor::new(
            vec![
                incubator_node.clone(),
                path_node
            ]
        );
        sum_editor.select(0);

        grid.insert_iter(
            vec![
                (Point2::new(0, 0), cwd_node.get_view()
                 .map_item(|_idx, x| x.add_style_back(nested::utils::color::fg_style_from_depth(1)))
                ),
                (Point2::new(1, 0), nested::terminal::make_label("$ ")),
                (Point2::new(3, 0), sum_editor.pty_view())
            ]
        );

        let product = nested::editors::product::ProductEditor::new(0, ctx.clone());

        let mut comp_port = ViewPort::new();
        let compositor = nested::terminal::TerminalCompositor::new(comp_port.inner());

        compositor.write().unwrap().push(
            incubator_node.get_edit::<PipelineLauncher>().unwrap()
                .read().unwrap()
                .box_port.outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((30, 80, 50))))
        );
        compositor.write().unwrap().push(
            grid.get_port().flatten()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((220, 220, 0)))),
        );
        
        Command {
            ctx: ctx.clone(),
            grid,
            cwd,
            cwd_node,
            comp_port,
            state: CommandState::Incubator(incubator_node.get_edit::<PipelineLauncher>().unwrap()),
            sum_editor: sum_editor.into_node(ctx)
        }
    }
}

impl ObjCommander for Command {
    fn send_cmd_obj(&mut self, obj: Arc<RwLock<ReprTree>>) {
        let cmd_obj = obj.clone();
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        let char_value =
            if cmd_type == (&self.ctx, "( Char )").into() {
                if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                    Some(cmd_view.get())
                } else {
                    None
                }
            } else {
                None
            };

        let term_event_value =
            if cmd_type == (&self.ctx, "( TerminalEvent )").into() {
                if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                    Some(cmd_view.get())
                } else {
                    None
                }
            } else {
                None
            };

        match self.state.clone() {
            CommandState::Incubator(mut incubator_editor) => {
                match char_value {
                    Some(' ') => {
                            let strings = incubator_editor.read().unwrap().get_strings();

                            let mut cd_cmd = false;

                            if strings.len() > 0 {
                                if strings[0].len() > 0 {
                                    if strings[0][0] == "cd" {
                                        cd_cmd = true;
                                    }
                                }
                            }

                            if cd_cmd {
                                self.grid.insert(Point2::new(2,0), nested::terminal::make_label("cd "));
                                incubator_editor.write().unwrap().editor.goto(TreeCursor::none());

                                let se = self.sum_editor.get_edit::<SumEditor>().unwrap();
                                let mut se = se.write().unwrap();

                                se.select(1);
                                let mut path_node = se.editors[1].clone();

                                self.state = CommandState::CD(path_node.clone());

                                path_node.goto(TreeCursor::home());
                            } else {
                                incubator_editor.write().unwrap().send_cmd_obj(obj);
                                self.state = CommandState::Pipeline(incubator_editor);
                            }
                    }
                    _ => {
                        self.sum_editor.send_cmd_obj(obj); 
                    }
                }
            }

            CommandState::CD(mut path) => {
                match term_event_value {
                    Some(TerminalEvent::Input(Event::Key(Key::Backspace))) => {
                        if path.get_cursor().tree_addr.iter().fold(
                            true,
                            |s, x| s && *x == 0
                        ) {
                            let se = self.sum_editor.get_edit::<SumEditor>().unwrap();
                            let mut se = se.write().unwrap();

                            let ed = se.editors[0].clone()
                                    .get_edit::<PipelineLauncher>().unwrap();

                            self.state = CommandState::Incubator(ed);
                            se.select(0);

                            self.grid.remove(Point2::new(2, 0));

                            se.goto(TreeCursor {
                                leaf_mode: ListCursorMode::Insert,
                                tree_addr: vec![ 0, 0, -1 ]
                            });
                        } else {
                            self.sum_editor.send_cmd_obj(obj.clone());
                        }
                    }
                    _ => {
                        match char_value {
                            Some('\n') => {
                                // todo set cwd
                                /*
                                let pwd_edit = self.ctx.write().unwrap()
                                    .get_obj("PWD")
                                    .get_edit::<ListEditor>().unwrap();
                                */
                            },
                            _ => {
                                self.sum_editor.send_cmd_obj(obj); 
                            }
                        }
                    }
                }

            }

            CommandState::Pipeline(mut pipeline) => {
                match char_value {
                    Some('\n') => {
                        pipeline.write().unwrap().launch();
                    },
                    _ => {
                        pipeline.write().unwrap().send_cmd_obj(obj);
                    }
                }
            }
        }
    }
}

