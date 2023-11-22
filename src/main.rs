extern crate portable_pty;
extern crate r3vi;

mod incubator;
mod pty;
mod path;
mod process;
mod pipeline;
mod command;
mod process_types;
mod interactive;

use {
    clap::{Parser},
    cgmath::{Point2, Vector2},
    r3vi::{
        view::{port::UpdateTask, Observer, ViewPort, AnyOuterViewPort, index::*, sequence::*, grid::GridWindowIterator},
        buffer::{singleton::*, vec::*, index_hashmap::*},
        projection::{decorate_sequence::*}
    },
    nested::{
        type_system::{Context, ReprTree},
        editors::{list::{ListCursorMode, PTYListController, PTYListStyle}},
        terminal::{make_label, Terminal, TerminalCompositor, TerminalEditor, TerminalEvent, TerminalStyle, TerminalProjections},
        tree::{TreeNav, TreeCursor},
        commander::ObjCommander
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    crate::pipeline::PipelineLauncher
};

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));
    crate::path::init_ctx(&mut ctx.write().unwrap());

    crate::process::ProcessLauncher::init_ctx(&mut ctx.write().unwrap());
    crate::pipeline::PipelineLauncher::init_ctx(&mut ctx.write().unwrap());
    crate::command::Command::init_ctx(&mut ctx.write().unwrap());

    {
        let mut c = ctx.write().unwrap();
        c.add_list_typename("None".into());
        c.add_list_typename("SepSeq".into());
        c.add_list_typename("FileInfo".into());
        c.add_list_typename("HumanizedDate".into());
        c.add_list_typename("Ctime".into());
        c.add_list_typename("Weekday".into());
        c.add_list_typename("Month".into());
        c.add_list_typename("LocaleShortWeekday".into());
        c.add_list_typename("LocaleFullWeekday".into());
        c.add_list_typename("LocaleShortMonth".into());
        c.add_list_typename("LocaleFullMonth".into());
    }
    
    ctx
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    check_expr: Option<String>,
}

#[async_std::main]
async fn main() {    
    let cli = Cli::parse();

    // Type Context //
    let ctx = Arc::new(RwLock::new(Context::default()));
    let ctx = init_os_ctx(ctx);

    if let Some(check_expr) = cli.check_expr.as_deref() {
        let mut node = Context::make_node(&ctx, Context::parse(&ctx, "Pipeline"), SingletonBuffer::new(0).get_port()).unwrap();
        
        node.goto(TreeCursor::home());
        for c in check_expr.chars() {
            node.send_cmd_obj(
                ReprTree::new_leaf(
                    Context::parse(&ctx, "Char"),
                    AnyOuterViewPort::from(SingletonBuffer::new(c).get_port())
                )
            );
        }

        let view_port = node.view.clone().unwrap();
        view_port.0.update();

        let pipeline = node.get_edit::<PipelineLauncher>().unwrap();
        let mut pipeline = pipeline.write().unwrap();
        pipeline.typecheck();

        let diagnostics_port = node.diag.unwrap();
        for message in diagnostics_port.get_view().unwrap().iter() {
            for x in message.addr.iter() {
                print!("{}.", x);
            }

            let view_port = message.port.clone();
            view_port.0.update();
            let view = view_port.get_view();

            match view.area() {
                IndexArea::Range(r) => {
                    let mut last_y = None;
                    for pos in GridWindowIterator::from(r) {

                        last_y = match last_y {
                            None => Some(pos.y),
                            Some(mut last_y) => {
                                while pos.y > last_y {
                                    last_y += 1;
                                    print!("\n");
                                }
                                Some(pos.y)
                            }
                        };
                        
                        if let Some(atom) = view.get(&pos) {
                            print!(
                                "{}{}",
                                atom.style,
                                atom.c.unwrap_or(' ')
                            );
                        } else {
                            print!(" ");
                        }
                    }
                }
                area => {
                    eprintln!("area: {:?}", area);
                }
            }

            println!("{}", termion::style::Reset);
        }

        println!("---");
    } else {
        interactive::tui_repl(ctx).await;
    }
}

