use {
    r3vi::{
        view::{
            OuterViewPort, ViewPort,
            singleton::*,
            sequence::*,
        }
    },
    nested::{
        terminal::{
            TerminalAtom, TerminalStyle,
            TerminalView,
            widgets::ascii_box::AsciiBox
        },
        tree::{NestedNode},
        editors::list::*,
        type_system::{Context, MorphismType, MorphismTypePattern, TypeTerm}
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::io::{Read, Write}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PipelineLauncher {
    pub editor: NestedNode,

    _ptybox: Arc<RwLock<AsciiBox>>,
    pub box_port: ViewPort<dyn TerminalView>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,

    comp_port: ViewPort<dyn TerminalView>,
    _compositor: Arc<RwLock<nested::terminal::TerminalCompositor>>,
}

impl PipelineLauncher {
    pub fn init_ctx(ctx: &mut Context) {
        ctx.add_list_typename("Pipeline".into());

        ctx.add_morphism(
            MorphismTypePattern {
                src_tyid: ctx.get_typeid("List"),
                dst_tyid: ctx.get_typeid("Pipeline").unwrap()
            },
            Arc::new(
                |mut node, _dst_type:_| {
                    let depth = node.depth;
                    let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                    let pty_editor = PTYListEditor::from_editor(
                        editor,
                        Some('|'),
                        depth
                    );

                    node.view = Some(pty_editor.pty_view((""," | ","")));
                    node.cmd = Some(Arc::new(RwLock::new(pty_editor)));

                    let pipeline_launcher = crate::pipeline::PipelineLauncher::new(node.clone());
                    
                    node.view = Some(pipeline_launcher.editor_view());

                    let editor = Arc::new(RwLock::new(pipeline_launcher));
                    node.cmd = Some(editor.clone());
                    node.editor = Some(editor.clone());

                    Some(node)                
                }
            )
        );

        ctx.add_node_ctor(
            "Pipeline",
            Arc::new(
                |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                    let mut node = Context::make_node(
                        &ctx,
                        (&ctx, "( List Process )").into(),
                        depth+1
                    ).unwrap();

                    node = node.morph(dst_typ);

                    Some(node)
                }
            )
        );
    }
    
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
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((30, 80, 50)))),
        );
        compositor.write().unwrap().push(
            editor.view.clone().unwrap()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((220, 220, 0)))),
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
            pty_port,
            suspended: false,
            comp_port,
            box_port,
            _compositor: compositor,
        }
    }

    pub fn clear_pty(&mut self) {
        
    }

    pub fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.box_port.outer()
    }

    pub fn editor_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor.get_view()
    }

    pub fn get_strings(&self) -> Vec<Vec<String>> {
        let ctx = self.editor.ctx.clone().unwrap();

        let mut pipeline_strings = Vec::<Vec<String>>::new();

        let pipeline_view = self.editor.get_data_view::<dyn SequenceView<Item = NestedNode>>(vec!["( List Process )"].into_iter());

        for i in 0..pipeline_view.len().unwrap_or(0) {
            let process_node = pipeline_view.get(&i).unwrap();
            let mut process_strings = Vec::<String>::new();

            let process_view = process_node.get_data_view::<dyn SequenceView<Item = NestedNode>>(vec!["( List ProcessArg )"].into_iter());

            for j in 0..process_view.len().unwrap_or(0) {
                let arg_node = process_view.get(&j).unwrap();
                let arg_view = arg_node.get_data_view::<dyn SequenceView<Item = NestedNode>>(vec!["( List Char )"].into_iter());
                
                let mut arg = String::new();
                for k in 0..arg_view.len().unwrap_or(0) {
                    let char_node = arg_view.get(&k).unwrap();
                    let char_view = char_node.get_data_view::<dyn SingletonView<Item = Option<char>>>(vec![].into_iter());
                    if let Some(c) = char_view.get() {
                        arg.push(c);
                    }
                }

                process_strings.push(arg);
            }

            pipeline_strings.push(process_strings);
        }

        pipeline_strings
    }

    pub fn launch(&mut self) {
        let strings = self.get_strings();

        let mut last_output : Option<std::process::ChildStdout> = None;

        for process_str in strings {
            if process_str.len() > 0 {
                let mut process = std::process::Command::new(process_str[0].clone());

                if last_output.is_some() {
                    process.stdin(std::process::Stdio::piped());
                }
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

        let _output_str = String::new();
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
        let _term_event_type = ctx.type_term_from_str("( TerminalEvent )").unwrap();
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

