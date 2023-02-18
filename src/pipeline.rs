use {
    crate::pty::{PTYStatus, PTY},
    r3vi::{
        view::{
            OuterViewPort, ViewPort,
            singleton::*,
            sequence::*,
        },
        projection::{
            filter_sequence::*,
            map_sequence::*
        }
    },
    nested::{
        editors::{
            list::{ListCursorMode, PTYListEditor},
            char::CharEditor,  
        },
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
            widgets::ascii_box::AsciiBox,
            make_label
        },
        tree::{TreeCursor, TreeNav, TreeNavResult, NestedNode},
        diagnostics::Diagnostics,
        type_system::{Context}
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::io::{Read, Write},
    termion::event::{Event, Key},
    cgmath::Vector2
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PipelineLauncher {
    editor: NestedNode,

    _ptybox: Arc<RwLock<AsciiBox>>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,

    comp_port: ViewPort<dyn TerminalView>,
    _compositor: Arc<RwLock<nested::terminal::TerminalCompositor>>,
}

impl PipelineLauncher {
    pub fn new(
        editor: NestedNode
    ) -> PipelineLauncher {
        let pty_port = ViewPort::new();
        let comp_port = ViewPort::new();
        let box_port = ViewPort::<dyn TerminalView>::new();
        let compositor = nested::terminal::TerminalCompositor::new(comp_port.inner());

        compositor.write().unwrap().push(
            box_port
                .outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((90, 120, 100)))),
        );
        compositor.write().unwrap().push(
            editor.view.clone().unwrap()
        );

        PipelineLauncher {
            editor,
            _ptybox: AsciiBox::new(
                cgmath::Vector2::new(0, 0),
                pty_port.outer().map_item(|_, a: &TerminalAtom| {
                    a.add_style_back(TerminalStyle::fg_color((230, 230, 230)))
                }),
                box_port.inner(),
            ),
            suspended: false,
            pty_port,
            comp_port,
            _compositor: compositor,
        }
    }

    pub fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.comp_port.outer()
    }

    fn get_strings(&self) -> Vec<Vec<String>> {

        let ctx = self.editor.ctx.clone().unwrap();
        let ctx = ctx.read().unwrap();

        let mut pipeline_strings = Vec::<Vec<String>>::new();

        if let Some(pipeline_data) = self.editor.data.clone() {
            let pipeline_view = pipeline_data.read().unwrap()
                .descend(
                    &ctx.type_term_from_str("( List Process )").unwrap()
                )
                .unwrap()
                .read().unwrap()
                .get_view::<dyn SequenceView<Item = NestedNode>>();

            for i in 0..pipeline_view.len().unwrap_or(0) {
                let process_node = pipeline_view.get(&i).unwrap();

                let mut process_strings = Vec::<String>::new();

                let process_view = process_node.data
                    .clone().unwrap()
                    .read().unwrap()
                    .descend(
                        &ctx.type_term_from_str("( List ProcessArg )").unwrap()
                    )
                    .unwrap()
                    .read().unwrap()
                    .get_view::<dyn SequenceView<Item = NestedNode>>();

                for j in 0..process_view.len().unwrap_or(0) {
                    let arg_node = process_view.get(&j).unwrap();

                    let arg_view = arg_node.data
                        .clone().unwrap()
                        .read().unwrap()
                        .descend(
                            &ctx.type_term_from_str("( List Char )").unwrap()
                        )
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

                    process_strings.push(arg);
                }

                pipeline_strings.push(process_strings);
            }
        }

        pipeline_strings
    }

    fn launch(&mut self) {
        let strings = self.get_strings();

        let mut last_output : Option<std::process::ChildStdout> = None;

        for process_str in strings {
            if process_str.len() > 0 {
                let mut process = std::process::Command::new(process_str[0].clone());
                process.stdin(std::process::Stdio::piped());
                process.stdout(std::process::Stdio::piped());

                for i in 1..process_str.len() {
                    process.arg(process_str[i].clone());
                }

                if let Ok(child) = process.spawn( ) {

                    if let Some(mut last_output) = last_output.take() {
                        let mut s = String::new();

                        match last_output.read_to_string(&mut s) {
                            Err(why) => panic!("couldn't read stdout: {}", why),
                            Ok(_) => {},
                        }

                        match child.stdin.unwrap().write_all(s.as_bytes()) {
                            Err(why) => panic!("couldn't write to stdin: {}", why),
                            Ok(_) => {},
                        }
                    }

                    last_output = child.stdout;
                }
            }
        }

        let mut output_str = String::new();
        if let Some(mut last_output) = last_output.take() {
            //eprintln!("pipeline output: {}", output_str);

            let max_size = cgmath::Vector2::new(80, 40);

            let port = self.pty_port.inner();

            async_std::task::spawn_blocking(move || {
                nested::terminal::ansi_parser::read_ansi_from(&mut last_output, max_size, port);
            });
        }
    }    
}

use nested::type_system::ReprTree;
use nested::commander::ObjCommander;

impl ObjCommander for PipelineLauncher {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {

        let ctx = self.editor.ctx.clone().unwrap();
        let ctx = ctx.read().unwrap();

        let co = cmd_obj.read().unwrap();
        let cmd_type = co.get_type().clone();
        let term_event_type = ctx.type_term_from_str("( TerminalEvent )").unwrap();
        let char_type = ctx.type_term_from_str("( Char )").unwrap();


        if cmd_type == char_type {
            if let Some(cmd_view) = co.get_view::<dyn SingletonView<Item = char>>() {
                drop(co);
                let c = cmd_view.get();
                
                if c == '\n' {
                    self.launch();
                } else {
                    self.editor.send_cmd_obj(cmd_obj);
                }

            } else {
                drop(co);
                self.editor.send_cmd_obj(cmd_obj);
            }            
        } else {
            drop(co);
            self.editor.send_cmd_obj(cmd_obj);            
        }
    }
}

