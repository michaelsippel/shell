extern crate portable_pty;
extern crate r3vi;

mod pty;
mod incubator;

// TODO rewrite process & command with incubator rules
//mod process;
//mod command;

use {
    cgmath::{Point2, Vector2},
    r3vi::{
        view::{
            port::UpdateTask, Observer, ViewPort,
            index::*,
        },
        buffer::{
            vec::*,
            singleton::*,
            index_hashmap::*
        },
        projection::{
            decorate_sequence::*
        }
    },
    nested::{
        type_system::{Context, ReprTree},
        editors::{
            list::{ListCursorMode, PTYListEditor, ListStyle},
            integer::{PosIntEditor},
            sum::*
        },
        terminal::{
            make_label, Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalProjections
        },
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics},
        commander::Commander
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

macro_rules! typeterm {
    ($ctx:ident, $s:expr) => { $ctx.read().unwrap().type_term_from_str($s).unwrap() }
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
    let ctx = nested::type_system::init_os_ctx(ctx);

/*
    let vb = VecBuffer::<char>::new();
    let rt_charvec = ReprTree::new_leaf(
        typeterm!(ctx, "( Vec Char )"),
        AnyOuterViewPort::from(vb.get_port())
    );

    let rt_typeid = ReprTree::ascend(&rt_char, typeterm!(ctx, "( Typeid )"));
    rt_typename.write().unwrap().insert_branch(
        ReprTree::new_leaf(
            typeterm!(ctx, "( MachineInt )"),
            AnyOuterViewPort::from(
                vb.get_port().to_sequence().map(
                    |c: &char| {
                        c.to_digit(10).unwrap()
                    }
                )
            )
        )
    );
*/
    let c = ctx.clone();

    let mut process_list_editor =
        PTYListEditor::new(
            ctx.clone(),
            c.read().unwrap().type_term_from_str("( TypeTerm )").unwrap(),
            ListStyle::VerticalSexpr,
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
            (Point2::new(0, 4), node.view.clone().unwrap()),
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
                    let mut c = node.get_cursor();
                    c.leaf_mode = match c.leaf_mode {
                        ListCursorMode::Select => ListCursorMode::Insert,
                        ListCursorMode::Insert => ListCursorMode::Select
                    };
                    node.goto(c);
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

