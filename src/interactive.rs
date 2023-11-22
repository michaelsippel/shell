
use {
    cgmath::{Point2, Vector2},
    r3vi::{
        view::{port::UpdateTask, Observer, ViewPort, AnyOuterViewPort, index::*},
        buffer::{singleton::*, vec::*, index_hashmap::*},
        projection::{decorate_sequence::*}
    },
    nested::{
        type_system::{Context, ReprTree},
        editors::{list::{ListCursorMode, ListEditor, PTYListController, PTYListStyle, ListCmd}, typeterm::TypeTermEditor},
        terminal::{make_label, Terminal, TerminalCompositor, TerminalEditor, TerminalEvent, TerminalStyle, TerminalProjections},
        tree::{TreeNav, TreeCursor},
        commander::ObjCommander
    },
    /* refactoring proposal
     *  nested-vt100:{}
     *  nested-terminal::{event_loop, display_loop}
     */
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

pub async fn tui_repl(ctx: Arc<RwLock<Context>>) {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    let portmutex = Arc::new(RwLock::new(()));

    // Update Loop //
    let tp = term_port.clone();

    async_std::task::spawn({
        let tp = term_port.clone();
        let portmutex = portmutex.clone();
        async move {
            loop {
                {
                    let _l = portmutex.write().unwrap();
                    tp.update();
                }
                async_std::task::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    });

    let mut process_list_editor =
        ListEditor::new(
            ctx.clone(),
            Context::parse(&ctx, "Command")
        );
    let ple_seg_view = PTYListStyle::new( ("", "", "") ).get_seg_seq_view( &mut process_list_editor );

    let cursor_widget = process_list_editor.get_cursor_widget();

    let mut node = process_list_editor.into_node( SingletonBuffer::new(0).get_port() );
    PTYListController::for_node( &mut node, None, None );

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
                         (Point2::new(0, 0), make_label(&format!("[{}]", n))),
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
            .push(table.get_port().flatten());

        node.goto(TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        });

    async_std::task::spawn(async move {
        tp.update();
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

            match node.get_cursor().leaf_mode {
                ListCursorMode::Select => {
                    match ev {
                        // left hand
                        TerminalEvent::Input(Event::Key(Key::Char('i'))) => { node.qpxev(); },
                        TerminalEvent::Input(Event::Key(Key::Char('a'))) => { node.dn(); },
                        TerminalEvent::Input(Event::Key(Key::Char('e'))) => { node.pxev(); },
                        TerminalEvent::Input(Event::Key(Key::Char('l'))) => { node.up(); },

                        // right hand
                        TerminalEvent::Input(Event::Key(Key::Char('n'))) => { node.nexd(); },
                        TerminalEvent::Input(Event::Key(Key::Char('r'))) => {
                            node.dn();
                            let mut c = node.get_cursor();
                            let d = c.tree_addr.len();
                            c.tree_addr[ d - 1 ] = -1;
                            node.goto(c);
                        },
                        TerminalEvent::Input(Event::Key(Key::Char('t'))) => { node.qnexd(); },
                        TerminalEvent::Input(Event::Key(Key::Char('g'))) => { node.up(); },

                        TerminalEvent::Input(Event::Key(Key::Char('\t'))) => { node.toggle_leaf_mode(); }

                        TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                            node.send_cmd_obj(
                                ReprTree::from_char(&ctx, c)
                            );
                        }

                       _ => {}
                    }
                }
                ListCursorMode::Insert => {
                    match ev {
                        TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                        // left hand
                        TerminalEvent::Input(Event::Key(Key::Ctrl('i'))) => { node.qpxev(); },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('a'))) => { node.dn(); },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('e'))) => { node.pxev(); },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('l'))) => { node.up(); },

                        // right hand
                        TerminalEvent::Input(Event::Key(Key::Ctrl('n'))) => { node.nexd(); },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('r'))) => {
                            node.pxev();
                            node.dn();
                            let mut c = node.get_cursor();
                            let d = c.tree_addr.len();
                            c.tree_addr[ d - 1 ] = -1;
                            node.goto(c);
                        },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('t'))) => { node.qnexd(); },
                        TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) => { node.up(); node.nexd(); },


                        // default cross
                        TerminalEvent::Input(Event::Key(Key::Left)) => { node.pxev(); }
                        TerminalEvent::Input(Event::Key(Key::Right)) => { node.nexd(); }
                        TerminalEvent::Input(Event::Key(Key::Up)) => { node.up(); }
                        TerminalEvent::Input(Event::Key(Key::Down)) => { node.dn(); }
                        TerminalEvent::Input(Event::Key(Key::PageUp)) => { node.up(); node.nexd(); }
                        TerminalEvent::Input(Event::Key(Key::PageDown)) => { node.pxev(); node.dn(); node.qnexd(); }
                        TerminalEvent::Input(Event::Key(Key::Home)) => { node.qpxev(); }
                        TerminalEvent::Input(Event::Key(Key::End)) => { node.qnexd(); }

                        TerminalEvent::Input(Event::Key(Key::Char('\t'))) => { node.toggle_leaf_mode(); }
                        TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                            node.send_cmd_obj(ListCmd::DeletePxev.into_repr_tree(&ctx));
                        },
                        TerminalEvent::Input(Event::Key(Key::Delete)) => {
                            node.send_cmd_obj(ListCmd::DeleteNexd.into_repr_tree(&ctx));
                        }

                        TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                            node.send_cmd_obj(
                                ReprTree::from_char(&ctx, c)
                            );
                        }

                        ev => {                    
                            node.handle_terminal_event(&ev);
                        }
                    }
                }
            }

            tp.update();
        }

        drop(term);
        drop(term_port);
    });

    term_writer.show().await.expect("output error!");
}

