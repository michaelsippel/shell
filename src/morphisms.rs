use {
    nested::{
        type_system::{Context, TypeTerm},
        editors::{
            list::*
        },
        type_system::{MorphismTypePattern},
    },
    std::sync::{Arc, RwLock},
};

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));
    
    ctx.write().unwrap().add_list_typename("PathSegment".into());
    let pattern = MorphismTypePattern {
        src_type: ctx.read().unwrap().type_term_from_str("( List Char )"),
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
        src_type: ctx.read().unwrap().type_term_from_str("( List PathSegment )"),
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

                node.view = Some(pty_editor.pty_view(
                    (
                        "<".into(),
                        "/".into(),
                        ">".into()
                    )
                ));
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
        src_type: ctx.read().unwrap().type_term_from_str("( List Char )"),
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
        src_type: ctx.read().unwrap().type_term_from_str("( List ProcessArg )"),
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
        src_type: ctx.read().unwrap().type_term_from_str("( List Process )"),
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

                node.view = Some(pty_editor.pty_view(
                    (
                        "$(".into(),
                        " | ".into(),
                        ")".into()
                    )
                ));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));


                let pipeline_launcher = crate::pipeline::PipelineLauncher::new(node.clone());
                node.view = Some(pipeline_launcher.pty_view());
                node.cmd = Some(
                    Arc::new(RwLock::new(pipeline_launcher))
                );


                Some(node)                
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "Pipeline", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Process").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );
    
    
    ctx
}


