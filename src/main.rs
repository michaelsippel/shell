extern crate portable_pty;
extern crate r3vi;

mod incubator;
mod pty;
mod path;
mod process;
mod pipeline;
mod command;

use {
    cgmath::{Point2, Vector2},
    r3vi::{
        view::{port::UpdateTask, Observer, ViewPort, AnyOuterViewPort, index::*},
        buffer::{singleton::*, vec::*, index_hashmap::*},
        projection::{decorate_sequence::*}
    },
    nested::{
        type_system::{Context, ReprTree},
        editors::{list::{ListCursorMode, PTYListEditor}},
        terminal::{make_label, Terminal, TerminalCompositor, TerminalEditor, TerminalEvent, TerminalStyle, TerminalProjections},
        tree::{TreeNav, TreeCursor},
        commander::ObjCommander
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    crate::path::init_ctx(&mut ctx.write().unwrap());
    crate::process::ProcessLauncher::init_ctx(&mut ctx.write().unwrap());
    crate::pipeline::PipelineLauncher::init_ctx(&mut ctx.write().unwrap());
    crate::command::Command::init_ctx(&mut ctx.write().unwrap());

    ctx
}

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    let portmutex = Arc::new(RwLock::new(()));

    // Update Loop //
    let tp = term_port.clone();
    async_std::task::spawn({
        let portmutex = portmutex.clone();
        async move {
            loop {
                {
                    let _l = portmutex.write().unwrap();
                    tp.update();
                }
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    });

    // Type Context //
    let ctx = Arc::new(RwLock::new(Context::new()));
    let ctx = nested::type_system::init_mem_ctx(ctx);
    let ctx = nested::type_system::init_editor_ctx(ctx);
    let ctx = nested::type_system::init_math_ctx(ctx);
    let ctx = init_os_ctx(ctx);

    let c = ctx.clone();

    let process_list_editor =
        PTYListEditor::new(
            ctx.clone(),
            (&ctx, "( Command )").into(),
            None,
            0
        );

    let ple_seg_view = process_list_editor.get_seg_seq_view();
    let cursor_widget = process_list_editor.editor.read().unwrap().get_cursor_widget();
    let mut node = process_list_editor.into_node();
    node = node.set_ctx(ctx.clone());

    async_std::task::spawn(async move {
        let mut table = IndexBuffer::new();

        let magic =
            make_label("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>")
            .map_item(|pos, atom| {
                atom.add_style_back(TerminalStyle::fg_color((
                    5,
                    ((80 + (pos.x * 30) % 100) as u8),
                    (55 + (pos.x * 15) % 180) as u8,
                )))
            });

        let mut cur_size = SingletonBuffer::new(Vector2::new(10, 10));

        table.insert_iter(vec![
            (Point2::new(0, 0), magic.clone()),
            (Point2::new(0, 1), cursor_widget),
            (Point2::new(0, 2), magic.clone()),
            (Point2::new(0, 3), make_label(" ")),
            (Point2::new(0, 4),
             ple_seg_view
             .enumerate()
             .map(
                 |(n, segment)| {
                     let mut buf = IndexBuffer::new();
                     buf.insert_iter(vec![
                         (Point2::new(0, 0),
                          make_label(match n+1 {
                              1 => "I) ",
                              2 => "II) ",
                              3 => "III) ",
                              4 => "IV) ",
                              5 => "V) ",
                              6 => "VI) ",
                              7 => "VII) ",
                              8 => "IIX) ",
                              9 => "IX) ",
                              10 => "X) ",
                              _ => ""
                          })),
                         (Point2::new(1, 0), segment.clone())
                     ]);

                     buf.get_port()
                 }
             )
             .separate({
                 let mut buf = IndexBuffer::new();
                 buf.insert(Point2::new(1,0), make_label(" ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~").with_fg_color((40,40,40))
                 );
                 buf.get_port()
             })
             .to_grid_vertical()
             .flatten()
             .flatten()
            ),

            (Point2::new(0, 5), make_label(" ")),
            (Point2::new(0, 6), magic.clone()),
            (Point2::new(0, 7), node.get_diag().map(
                |entry| {
                    let mut b = VecBuffer::new();
                    b.push(
                         make_label("@").with_style(
                             TerminalStyle::bold(true)
                                 .add(TerminalStyle::fg_color((120,120,0))))
                    );

                    for x in entry.addr.iter() {
                        b.push(
                            make_label(&format!("{}", x)).with_fg_color((0, 100, 20))
                        );
                        b.push(
                            make_label(".")
                                .map_item(|_p,a| a
                                   .add_style_back(TerminalStyle::bold(true))
                                   .add_style_back(TerminalStyle::fg_color((120,120,0))))
                        );
                    }

                    b.push(entry.port.clone());
                    b.get_port()
                        .to_sequence()
                        .to_grid_horizontal()
                        .flatten()
                        .map_item(move |_p,a| {
                            let select = false;
                            if select {
                                a.add_style_back(TerminalStyle::fg_color((60,60,60)))
                            } else {
                                *a
                            }
                        })
                }
            ).to_grid_vertical().flatten())
        ]);
        
        let (_w, _h) = termion::terminal_size().unwrap();

        compositor
            .write()
            .unwrap()
            .push(table.get_port().flatten().offset(Vector2::new(3, 0)));

        node.goto(TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        });

        loop {
            let ev = term.next_event().await;
            let _l = portmutex.write().unwrap();

            if let TerminalEvent::Resize(new_size) = ev {
                cur_size.set(new_size);
                term_port.inner().get_broadcast().notify(&IndexArea::Full);
                continue;
            }
/*
            if let Some(process_editor) = process_list_editor.get_item() {
                let mut pe = process_editor.write().unwrap();
                /*
                if pe.is_captured() {
                    if let TerminalEditorResult::Exit = pe.handle_terminal_event(&ev) {
                        drop(pe);
                        process_list_editor.up();
                        process_list_editor.nexd();
                    }
                    continue;
            }
                */
            }
*/
            match ev {
                TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,
                TerminalEvent::Input(Event::Key(Key::Ctrl('l'))) => {
                    node.goto(TreeCursor {
                        leaf_mode: ListCursorMode::Insert,
                        tree_addr: vec![0],
                    });
                    //process_list_editor.clear();
                }
                TerminalEvent::Input(Event::Key(Key::Left)) => {
                    node.pxev();
                }
                TerminalEvent::Input(Event::Key(Key::Right)) => {
                    node.nexd();
                }
                
                TerminalEvent::Input(Event::Key(Key::Up)) => {
                    node.up();
                }
                TerminalEvent::Input(Event::Key(Key::Down)) => {
                    node.dn();
                }

                TerminalEvent::Input(Event::Key(Key::PageUp)) => {
                    node.up();
                    node.nexd();
                }
                TerminalEvent::Input(Event::Key(Key::PageDown)) => {
                    node.pxev();
                    node.dn();
                    node.qnexd();
                }

                TerminalEvent::Input(Event::Key(Key::Home)) => {
                    node.qpxev();
                }
                TerminalEvent::Input(Event::Key(Key::End)) => {
                    node.qnexd();
                }
                TerminalEvent::Input(Event::Key(Key::Char('\t'))) => {
                    node.toggle_leaf_mode();
                }
                TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                    let buf = SingletonBuffer::new(c);

                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                            AnyOuterViewPort::from(buf.get_port())
                        )
                    );
                }
                ev => {                    
                    node.handle_terminal_event(&ev);
                }
            }
        }

        drop(term);
        drop(term_port);
    });

    term_writer.show().await.expect("output error!");
}

