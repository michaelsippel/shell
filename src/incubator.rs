
use {
    std::sync::{Arc, RwLock},
    r3vi::{
        view::{
            AnyOuterViewPort,
            singleton::*
        },
        buffer::{
            singleton::*,
            vec::*
        }
    },
    nested::{
        commander::Commander,
        terminal::TerminalEvent,
        type_system::{TypeTerm, ReprTree, Context},
    },
    termion::event::{Event, Key},
};

struct Incubator {
    stack: VecBuffer<Arc<RwLock<ReprTree>>>,
    target: TypeTerm,
    ctx: Arc<RwLock<Context>>
}

impl Incubator {
    pub fn new(ctx: Arc<RwLock<Context>>, target: TypeTerm) -> Self {
        Incubator {
            ctx,
            target,
            stack: VecBuffer::new()
        }
    }

    fn incubate(&self, obj: Arc<RwLock<ReprTree>>) -> Option<Arc<RwLock<ReprTree>>> {
        let c = self.ctx.read().unwrap();
        let o = obj.read().unwrap();

        if o.get_type() == &c.type_term_from_str("( TerminalEvent )").unwrap() {
            let cmd = o.get_port::<dyn SingletonView<Item = TerminalEvent>>().unwrap().get_view().unwrap().get();
            match cmd {
                TerminalEvent::Input(Event::Key(Key::Char(ch))) => {
                    return Some(ReprTree::new_leaf(
                            c.type_term_from_str("( Char )").unwrap(),
                            AnyOuterViewPort::from(
                                SingletonBuffer::new(ch).get_port()
                            )
                        ));
                },
                _ => {}
            }
        }

        if o.get_type() == &c.type_term_from_str("( Char )").unwrap() {
            let ch = o.get_port::<dyn SingletonView<Item = char>>().unwrap().get_view().unwrap().get();

            if ch.is_digit(10) {
                return Some(ReprTree::ascend(
                    &obj,
                    c.type_term_from_str("( Digit 10 )").unwrap()
                ));
            }
        }

        None
    }
}

impl Commander for Incubator {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        self.stack.push(
            ReprTree::new_leaf(
                            self.ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                            AnyOuterViewPort::from(
                                SingletonBuffer::new(event.clone()).get_port()
                            )
                        )
        );
    }
}

