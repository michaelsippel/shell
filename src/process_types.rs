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

#[derive(Clone)]
pub struct PipeTypes {
    stdin_type: Option<TypeLadder>,
    stdout_type: Option<TypeLadder>
}

pub struct ProcessTypes {
    pipe_types: HashMap< Vec<String>, PipeTypes >
}

impl ProcessTypes {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        let mut pipe_types = HashMap::new();
        
        pipe_types.insert(
            vec!["wc".into(), "-l".into()],
            PipeTypes {
                stdin_type: Some(vec![
                    (&ctx, "( Sequence ( Sequence Char ) )").into(),
                ].into()),
                stdout_type: Some(vec![
                    (&ctx, "( ℕ )").into(),
                    (&ctx, "( PosInt 10 BigEndian )").into(),
                    (&ctx, "( Sequence ( Digit 10 ) )").into(),
                    (&ctx, "( Sequence Char )").into(),
                ].into())
            }
        );

        pipe_types.insert(
            vec!["sort".into(), "-n".into()],
            PipeTypes {
                stdin_type: Some(vec![
                    (&ctx, "( Sequence ( Sequence Char ) )").into(),
                ].into()),
                stdout_type: Some(vec![
                    (&ctx, "( Sequence ( Sequence Char ) )").into(),
                ].into())
            }
        );

        pipe_types.insert(
            vec!["xargs".into(), "expr".into(), "2".into(), "+".into()],
            PipeTypes {
                stdin_type: Some(vec![
                    (&ctx, "( ℕ )").into(),
                    (&ctx, "( PosInt 10 BigEndian )").into(),
                    (&ctx, "( Sequence ( Digit 10 ) )").into(),
                    (&ctx, "( Sequence Char )").into(),
                ].into()),
                stdout_type: Some(vec![
                    (&ctx, "( ℕ )").into(),
                    (&ctx, "( PosInt 10 BigEndian )").into(),
                    (&ctx, "( Sequence ( Digit 10 ) )").into(),
                    (&ctx, "( Sequence Char )").into(),
                ].into())
            }
        );

        pipe_types.insert(
            vec!["date".into(), "+%s".into()],
            PipeTypes {
                stdin_type: None,
                stdout_type: Some(vec![
                    (&ctx, "( Date )").into(),
                    (&ctx, "( TimeSinceEpoch )").into(),
                    (&ctx, "( Duration Seconds )").into(),
                    (&ctx, "( ℕ )").into(),
                    (&ctx, "( PosInt 10 BigEndian )").into(),
                    (&ctx, "( Sequence ( Digit 10 ) )").into(),
                    (&ctx, "( Sequence Char )").into(),
                ].into())
            }
        );

        pipe_types.insert(
            vec!["xargs".into(), "stat".into(), "-c".into(), "%W".into()],
            PipeTypes {
                stdin_type: Some(vec![
                    (&ctx, "( Sequence Path )").into(),
                    (&ctx, "( Sequence ( Sequence PathSegment ) )").into(),
                    (&ctx, "( Sequence ( Sequence ( Sequence Char ) ) )").into(),
                ].into()),
                stdout_type: Some(vec![
                    (&ctx, "( Date )").into(),
                    (&ctx, "( TimeSinceEpoch )").into(),
                    (&ctx, "( Duration Seconds )").into(),
                    (&ctx, "( ℕ )").into(),
                    (&ctx, "( PosInt 10 BigEndian )").into(),
                    (&ctx, "( Sequence ( Digit 10 ) )").into(),
                    (&ctx, "( Sequence Char )").into(),
                ].into())
            }
        );

        pipe_types.insert(
            vec!["find".into(), "src".into()],
            PipeTypes {
                stdin_type: None,
                stdout_type: Some(vec![
                    (&ctx, "( Sequence Path )").into(),
                    (&ctx, "( Sequence ( Sequence PathSegment ) )").into(),
                    (&ctx, "( Sequence ( Sequence ( Sequence Char ) ) )").into(),
//                    (&ctx, "( Sequence ( SeparatedSeq Char / ) )").into(),
                    (&ctx, "( Sequence ( Sequence Char ) )").into(),
//                    (&ctx, "( SeparetedSeq Char \n )").into(),
                ].into())
            }
        );

        ProcessTypes {
            pipe_types
        }
    }

    pub fn get_stdout_type(&self, cmd: &Vec<String>) -> Option<TypeLadder> {
        self.pipe_types.get(cmd)?.stdout_type.clone()
    }
    pub fn get_stdin_type(&self, cmd: &Vec<String>) -> Option<TypeLadder> {
        self.pipe_types.get(cmd)?.stdin_type.clone()
    }
}

