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
                PTYListController::for_node( &mut node, None, None );
                PTYListStyle::for_node( &mut node, ("","","") );
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
                PTYListController::for_node( &mut node, Some('/'), None );
                PTYListStyle::for_node( &mut node, ("","/","") );
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

