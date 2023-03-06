use {
    std::sync::{Arc, RwLock},
    nested::{
        type_system::{Context, TypeTerm, MorphismTypePattern},
        editors::list::*
    }
};

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_list_typename("PathSegment".into());
    ctx.add_morphism(
        MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("PathSegment").unwrap()
        },
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().get_view().unwrap().get().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
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

    ctx.add_node_ctor(
        "PathSegment",
        Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_fun_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap()).into()
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

    ctx.add_list_typename("Path".into());
    ctx.add_morphism(
        MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("Path").unwrap()
        },
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().get_view().unwrap().get().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    Some('/'),
                    depth+1
                );

                node.view = Some(pty_editor.pty_view(("","/","")));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)                
            }
        )
    );

    ctx.add_node_ctor(
        "Path", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_fun_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("PathSegment").unwrap()).into()
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

}

