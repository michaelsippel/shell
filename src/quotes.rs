use {
    std::sync::{Arc, RwLock},
    laddertypes::{TypeTerm},
    nested::{
        type_system::{Context, MorphismTypePattern},
        editors::list::*
    }
};

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_list_typename("shell::SingleQuote".into());
    ctx.add_morphism(
        MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("shell::SingleQuote").unwrap()
        },
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, None );
                PTYListStyle::for_node( &mut node, ("\'","","\'") );
                Some(node)
            }
        )
    );

    ctx.add_node_ctor(
        "shell::SingleQuote",
        Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth| {
                let mut node = Context::make_node( &ctx, Context::parse(&ctx, "<List Char>"), depth ).unwrap();
                node = node.morph(dst_typ);
                Some(node)
            }
        )
    );



    ctx.add_list_typename("shell::DoubleQuote".into());
    ctx.add_morphism(
        MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("shell::DoubleQuote").unwrap()
        },
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, None );
                PTYListStyle::for_node( &mut node, ("\"","","\"") );
                Some(node)
            }
        )
    );

    ctx.add_node_ctor(
        "shell::DoubleQuote",
        Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth| {
                let mut node = Context::make_node( &ctx, Context::parse(&ctx, "<List Char>"), depth ).unwrap();
                node = node.morph(dst_typ);
                Some(node)
            }
        )
    );
}

