
#[derive(Editor)]
struct PathSegment(Vec<Char>);

#[derive(Editor)]
struct Path(
    #[infix "/"]
    Vec<PathSegment>
);

#[derive(Editor)]
enum ExprSegment {
    Literal(Vec<char>),

    #[prefix "$"]
    Substitution(Substitution),

    #[wrap "\"" "\""]
    DoubleQuote(Vec<ExprSegment>),
}

#[derive(Editor)]
enum Substitution {
    Variable(String),

    #[wrap "(" ")"]
    Subshell(Command)
}

struct Process {
    executable: Path,

    #[space 1]
    #[infix " "]
    args: Vec<ExprSegment>
}

enum Command {
    #[prefix "cd "]
    Cd,

    Process(Process),

    Pipeline(
        #[infix "|"]
        Vec<Process>
    ),
    OnSucc(
        #[infix "&&"]
        Vec<Command>
    ),
    OnErr(
        #[infix "||"]
        Vec<Command>
    )
}

