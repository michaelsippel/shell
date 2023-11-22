
struct IEEEArgs {
    options: Vec<Vec<String>>,
    operands: Vec<String>
}

impl IEEEArgs {
    pub fn get_option(&self, opt: String) -> Vec<String> {
        for o in self.options.iter() {
            if o[0] == opt {
                return o.clone();
            }
        }
    }
}

pub enum IEEEArgPatternAtom {
    Var( String ),
    Lit( String ),
}

pub struct IEEEArgPattern {
    option: Vec< String, IEEEArgPatternAtom >,
    operands: Vec< IEEEArgPatternAtom >
}

pub enum CommandPattern {
    IEEE(IEEEArgPattern),
    RegExp(String)
}

// Assigns each variable name a type
pub struct Substitution(HashMap<String, TypeTerm>);

impl CommandPattern {
    pub fn match_pattern(&self, argv: Vec<String>) -> Option<Substitution> {
        
    }
}

