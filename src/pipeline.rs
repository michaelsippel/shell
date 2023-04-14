use {
    r3vi::{
        view::{
            OuterViewPort, ViewPort,
            singleton::*,
            sequence::*,
            port::UpdateTask
        },
        buffer::{
            vec::*,
            index_hashmap::*
        }
    },
    nested::{
        terminal::{
            TerminalAtom, TerminalStyle,
            TerminalView,
            widgets::ascii_box::AsciiBox,
            make_label,
            TerminalProjections
        },
        tree::{NestedNode, TreeNavResult},
        editors::list::*,
        type_system::{Context, MorphismType, MorphismTypePattern, TypeTerm, TypeLadder}
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::io::{Read, Write},

    cgmath::{Point2, Vector2},

    crate::process_types::ProcessTypes
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PipelineLauncher {
    pub editor: NestedNode,
    pub cwd: Option<String>,

    pub types: Arc<RwLock<ProcessTypes>>,

    _ptybox: Arc<RwLock<AsciiBox>>,
    pub box_port: ViewPort<dyn TerminalView>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,

    typegrid: IndexBuffer<Point2<i16>, OuterViewPort<dyn TerminalView>>,
    typeinfo_port: OuterViewPort<dyn TerminalView>,

    diag_buf: VecBuffer<nested::diagnostics::Message>,

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
                    PTYListController::for_node( &mut node, Some('|'), None );
                    PTYListStyle::for_node( &mut node, (""," | ","") );

                    let pipeline_launcher = crate::pipeline::PipelineLauncher::new(node.clone());

                    node.view = Some(pipeline_launcher.editor_view());
                    node.diag = Some(pipeline_launcher.diag_buf
                                     .get_port()
                                     .to_sequence());

                    let editor = Arc::new(RwLock::new(pipeline_launcher));
                    node.cmd = Some(editor.clone());
                    node.editor = Some(r3vi::buffer::singleton::SingletonBuffer::new(Some(editor.clone() as Arc<dyn std::any::Any + Send + Sync>)).get_port());
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

        let mut typegrid = IndexBuffer::new();
        let typeinfo_port = typegrid.get_port().flatten();

        let mut diag_buf = VecBuffer::new();

        compositor.write().unwrap().push(
            box_port
                .outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((30, 80, 50)))),
        );
        compositor.write().unwrap().push(
            editor.view.clone().unwrap()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((220, 220, 0)))),
        );

        let ctx = editor.ctx.clone().unwrap();

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
            cwd: None,
            _compositor: compositor,

            typegrid,
            types: Arc::new(RwLock::new(ProcessTypes::new(ctx))),
            typeinfo_port,

            diag_buf
        }
    }

    pub fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.comp_port.outer()
    }

    pub fn editor_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor.get_view()
    }

    pub fn get_type_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.typeinfo_port.clone()
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

                if arg.len() > 0 {
                    process_strings.push(arg);
                }
            }

            if process_strings.len() > 0 {
                pipeline_strings.push(process_strings);
            }
        }

        pipeline_strings
    }

    pub fn typecheck(&mut self) -> bool {
        let strings = self.get_strings();

        if strings.len() == 0 {
            self.diag_buf.push(nested::diagnostics::make_warn(
                make_label("empty pipeline")
            ));
            return false;
        }

        let ctx = self.editor.ctx.clone().unwrap();

        let types = self.types.read().unwrap();
        let mut last_stdout_type : Option<TypeLadder> = None;

        let types = self.types.read().unwrap();
        let mut last_stdout_type : Option<TypeLadder> = None;


        let mut typestack = vec![];

        for (j, process_str) in strings.iter().enumerate() {
            if process_str.len() > 0 {
                if let (Some(last_stdout), Some(expected)) = (last_stdout_type, types.get_stdin_type( &process_str )) {

                    // todo
                    match last_stdout.is_matching_repr(&expected) {
                        Ok(x) => {
                            let mut grid = IndexBuffer::new();
                            grid.insert(Point2::new(0 as i16, 0 as i16), make_label("matching types. ").with_style(TerminalStyle::bold(true)));
                            grid.insert(Point2::new(0 as i16, 1 as i16), make_label("found").with_style(TerminalStyle::bold(true)));

                            for (i,t) in last_stdout.0.iter().enumerate() {
                                if i < x {
                                    typestack.push(t.clone());
                                }

                                let tstr = ctx.read().unwrap().type_term_to_str( t );
                                grid.insert(Point2::new(0, 2+i as i16), make_label(&tstr).with_fg_color(
                                    if i < x {
                                        (120,120,120)
                                    } else {
                                        (160, 160, 20)
                                    }
                                ));
                            }

                            grid.insert(Point2::new(2, 1), make_label("expected").with_style(TerminalStyle::bold(true)));
                            grid.insert(Point2::new(1, 2 as i16 + x as i16), make_label("<===>").map_item(|x,a| a.add_style_back(TerminalStyle::fg_color((50,200,50)))));

                            for (i,t) in expected.0.iter().enumerate() {
                                let tstr = ctx.read().unwrap().type_term_to_str( t );
                                grid.insert(Point2::new(2, 2 as i16 + x as i16 +i as i16), make_label(&tstr).with_fg_color((160,160,20)));
                            }

                            self.diag_buf.push({
                                let mut msg = nested::diagnostics::make_info(
                                    grid.get_port().flatten()
                                );
                                msg.addr.push(j);
                                msg
                            });
                        }
                        Err(x) => {

                            let (first_match, first_mismatch) = match x {
                                Some((first_match, first_mismatch)) => {
                                    (Some(first_match), Some(first_mismatch))
                                }
                                None => (None, None)
                            };

                            let mut grid = IndexBuffer::new();
                            grid.insert(Point2::new(0 as i16, 0 as i16), make_label("type error. ").with_style(TerminalStyle::bold(true)));
                            grid.insert(Point2::new(0 as i16, 1 as i16), make_label("found").with_style(TerminalStyle::bold(true)));

                            for (i,t) in last_stdout.0.iter().enumerate() {
                                let tstr = ctx.read().unwrap().type_term_to_str( t );
                                grid.insert(Point2::new(0, 2+i as i16), make_label(&tstr).with_fg_color((160,160,20)));
                            }

                            grid.insert(Point2::new(2, 1), make_label("expected").with_style(TerminalStyle::bold(true)));

                            grid.insert(Point2::new(1, 2 as i16 + first_match.unwrap_or(0) as i16 + first_mismatch.unwrap_or(0) as i16), make_label("<=!=>").with_fg_color((200,50,50)));

                            for (i,t) in expected.0.iter().enumerate() {
                                let tstr = ctx.read().unwrap().type_term_to_str( t );
                                grid.insert(Point2::new(2, 2 as i16 + first_match.unwrap_or(0) as i16 + i as i16), make_label(&tstr).with_fg_color((160,160,20)));
                            }

                            self.diag_buf.push({
                                let mut msg = nested::diagnostics::make_error(
                                    grid.get_port().flatten()
                                );
                                msg.addr.push(j);
                                msg
                            });                            

                            return false;
                        }
                        Err(None) => {
                            return false;
                        }
                    }
                } else if j > 0 {                    
                    self.diag_buf.push({
                        let mut msg = nested::diagnostics::make_warn(
                            make_label("could not check, missing typeinfo")
                        );
                        msg.addr.push(j);
                        msg
                    });
                }

                last_stdout_type = types.get_stdout_type( &process_str );
                /*
                if let Some(mut last_stdout_type) = last_stdout_type.as_mut() {
                    while let Some(x) = typestack.pop() {
                        last_stdout_type.0.insert(0, x);   
                    }
            }
                */
            }
        }

        self.diag_buf.push(nested::diagnostics::make_info(
            make_label("type check ok")
        ));

        true
    }

    pub fn launch(&mut self) {
        self.pty_reset();

        if self.typecheck()
        {
            let strings = self.get_strings();

            let mut execs = Vec::new();
            for process_str in strings {
                if process_str.len() > 0 {                 
                    let mut exec = subprocess::Exec::cmd(process_str[0].clone());

                    if let Some(cwd) = self.cwd.as_ref() {
                        exec = exec.cwd(cwd);
                    }

                    for i in 1..process_str.len() {
                        exec = exec.arg(process_str[i].clone());
                    }
                    execs.push(exec);
                }
            }

            if execs.len() > 1 {
                let pipeline = subprocess::Pipeline::from_exec_iter(execs);

                match pipeline.stream_stdout() {
                    Ok(mut stdout) => {
                        let max_size = cgmath::Vector2::new(80, 40);

                        let port = self.pty_port.inner();

                        async_std::task::spawn_blocking(move || {
                            nested::terminal::ansi_parser::read_ansi_from(&mut stdout, max_size, port);
                        });
                    }

                    Err(err) => {
                        self.diag_buf.push(
                        nested::diagnostics::make_error(
                            make_label(
                                &format!("error spawning pipeline: {:?}", err)
                            )
                        ));
                    }
                }
            } else if execs.len() == 1 {
                let e = execs[0].clone();
                match e.stream_stdout() {
                    Ok(mut stdout) => {
                        let max_size = cgmath::Vector2::new(80, 40);

                        let port = self.pty_port.inner();

                        async_std::task::spawn_blocking(move || {
                            nested::terminal::ansi_parser::read_ansi_from(&mut stdout, max_size, port);
                        });
                    }
                    Err(err) => {
                        self.diag_buf.push(
                            nested::diagnostics::make_error(
                                make_label(
                                    &format!("error spawning pipeline: {:?}", err)
                                )
                            ));
                    }
                }
            }
        }
    }

    pub fn pty_reset(&mut self) {
        self.diag_buf.clear();
        self.typegrid.clear();
        let mut empty = IndexBuffer::new();
        self.pty_port.set_view(empty.get_port().get_view());
    }
}

use nested::type_system::ReprTree;
use nested::commander::ObjCommander;

impl ObjCommander for PipelineLauncher {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {

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
                    TreeNavResult::Exit
                } else {
                    self.editor.send_cmd_obj(cmd_obj)
                }
            } else {
                drop(co);
                self.editor.send_cmd_obj(cmd_obj)
            }            
        } else {
            drop(co);
            self.editor.send_cmd_obj(cmd_obj)
        }
    }
}

