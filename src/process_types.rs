use {
    nested::{
        type_system::{
            Context, TypeLadder
        }
    },

    std::{
        sync::{Arc, RwLock},
        collections::HashMap
    }
};
/*
pub enum CommandArgPattern {
    Any,
    Char(char),
    Product(Vec<CommandArgPattern>),
    Sum(Vec<CommandArgPattern>),
    List(Box<CommandArgPattern>),
}

#[derive(PartialEq, Eq, Hash)]
enum ProcessItem {
    PipeIn(usize),
    PipeOut(usize),
    Arg(usize),
    Env(String)
}


CommandArgPattern::Product(vec![
    CommandArgPattern::Product(vec![
        CommandArgPattern::Char('w'),
        CommandArgPattern::Char('c')
    ]),
    CommandArgPattern::List(Box::new(CommandArgPattern::Any))
])

pub struct TypeAssignmentBlock {
    assignments: HashMap<ProcessItem, TypeLadder>
}

struct CommandArgParser {
    patterns: Vec<(CommandArgPattern, TypeAssignmentBlock)>    
}

impl CommandArgParser {
    /*
    fn parse(&self, s: &str) -> Option<TypeAssignmentBlock> {
        match self {
            CommandArgPattern::Any => {
                None
            }
            CommandArgPattern::Char(c) => {
                
            }
            CommandArgPattern::
        }
}
    */
}
*/

pub struct ProcessTypes {
    ctx: Arc<RwLock<Context>>,
}

impl ProcessTypes {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        ProcessTypes {
            ctx
        }
    }

    pub fn get_type(&self, cmd: &Vec<String>, item: &str) -> Option<TypeLadder> {
        let db = String::from(env!("CARGO_MANIFEST_DIR")) + "/typedb";
        let gt = String::from(env!("CARGO_MANIFEST_DIR")) + "/gettype.sh";
        let stdout_typeladder_str = std::process::Command::new(gt)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .arg(db)
            .arg(cmd.join(" "))
            .arg(item)
            .output()
            .ok()?
            .stdout;

        let stdout_typeladder_str = String::from_utf8(stdout_typeladder_str).ok()?;

        let mut typeladder = TypeLadder(vec![]);

        if stdout_typeladder_str.len() > 0 {
            for typestr in stdout_typeladder_str.split('~') {
                typeladder.0.push((&self.ctx, typestr).into());
            }

            if typeladder.0.len() > 0 {
                Some(typeladder)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_stdin_type(&self, cmd: &Vec<String>) -> Option<TypeLadder> {
        self.get_type(cmd, ">0")
    }
    
    pub fn get_stdout_type(&self, cmd: &Vec<String>) -> Option<TypeLadder> {
        self.get_type(cmd, "<1")
    }
}

