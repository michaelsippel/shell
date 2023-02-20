use {
    crate::command::Command,
    nested::{
        type_system::{Context, TypeTerm},
        editors::{list::*, product::*},
        type_system::{MorphismTypePattern, TypeLadder},
        tree::NestedNode,
        terminal::TerminalEditor,
        diagnostics::Diagnostics
    },
    std::sync::{Arc, RwLock},
    cgmath::Point2
};

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));
    
    ctx.write().unwrap().add_list_typename("PathSegment".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("PathSegment").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    None,
                    depth+1
                );

                node.view = Some(pty_editor.pty_view(("","","")));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "PathSegment", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );




    
    ctx.write().unwrap().add_list_typename("Path".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("Path").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    Some('/'),
                    depth+1
                );

                node.view = Some(pty_editor.pty_view(("<","/",">")));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)                
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "Path", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("PathSegment").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

        
    ctx.write().unwrap().add_list_typename("ProcessArg".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("ProcessArg").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    None,
                    depth+1
                );

                node.view = Some(pty_editor.pty_view(
                    (
                        "".into(),
                        "".into(),
                        "".into()
                    )
                ));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)                
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "ProcessArg", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );


    ctx.write().unwrap().add_list_typename("Process".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("Process").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    Some(' '),
                    depth+1
                );

                node.view = Some(pty_editor.pty_view( ("", " ", "") ));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));

                let _process_launcher = crate::process::ProcessLauncher::new(node.clone());
/*
                node.view = Some(
                    process_launcher.pty_view()
            );
                node.cmd = Some(
                    Arc::new(RwLock::new(process_launcher))
                );
*/
                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "Process", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("ProcessArg").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );



    ctx.write().unwrap().add_list_typename("Pipeline".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("Pipeline").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    Some('|'),
                    depth+1
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

    ctx.write().unwrap().add_node_ctor(
        "Pipeline", Arc::new(
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


    ctx.write().unwrap().add_list_typename("Command".into());    
    ctx.write().unwrap().add_node_ctor(
        "Command", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                /*
                let mut node = Context::make_node(
                    &ctx,
                    (&ctx, "( Pipeline )").into(),
                    depth+1
                ).unwrap();
                 */

                /*
                let mut editor = ProductEditor::new(depth+1, ctx.clone())
                    .with_n(Point2::new(0,0), vec![ (&ctx, "( Path )").into() ])
                    .with_t(Point2::new(1,0), "$ ")
                    .with_n(Point2::new(2,0), vec![ (&ctx, "( Pipeline )").into() ]);

                let view = editor.get_term_view();
                let diag = editor.get_msg_port();
                let editor = Arc::new(RwLock::new(editor));

                let node = NestedNode::new(depth)
                    .set_ctx(ctx)
                    .set_cmd(editor.clone())
                    .set_nav(editor.clone())
                    .set_view(view)
                    .set_diag(diag)
                ;

                 */

                let node = Command::new(ctx,
                                        std::env::current_dir().unwrap()
                                        .into_os_string().into_string().unwrap()).into_node();
                Some(node)
            }
        )
    );

    
    ctx
}


