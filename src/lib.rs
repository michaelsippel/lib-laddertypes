

pub mod lexer;
pub mod bimap;
pub mod dict;
pub mod term;

pub use {
    dict::*,
    term::*,
};

#[cfg(test)]
mod tests {
    #[test]
    fn test_lexer() {
        use crate::lexer::*;

        {
            let mut lex = LadderTypeLexer::new("symbol".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("symbol".into()))) );
            assert_eq!( lex.next(), None );
        }
        {
            let mut lex = LadderTypeLexer::new("1234".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(1234))) );
            assert_eq!( lex.next(), None );
        }
        {
            let mut lex = LadderTypeLexer::new("123xxx".chars());
            assert_eq!( lex.next(), Some(Err(LexError::InvalidDigit)) );
        }
        {
            let mut lex = LadderTypeLexer::new("'x'".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Char('x'))) );
            assert_eq!( lex.next(), None );
        }
        {
            let mut lex = LadderTypeLexer::new("'xx'".chars());
            assert_eq!( lex.next(), Some(Err(LexError::InvalidChar)) );
        }
        {
            let mut lex = LadderTypeLexer::new("abc~def".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
            assert_eq!( lex.next(), None );
        }
        {
            let mut lex = LadderTypeLexer::new("abc   ~ def".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
            assert_eq!( lex.next(), None );
        }

        {
            let mut lex = LadderTypeLexer::new("<Seq Char>".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)) );
            assert_eq!( lex.next(), None );
        }
        {
            let mut lex = LadderTypeLexer::new("   <Seq      Char  >".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)) );
            assert_eq!( lex.next(), None );
        }

        {
            let mut lex = LadderTypeLexer::new("<Seq Date~<TimeSince UnixEpoch>~<Duration Seconds>~ℕ~<PosInt 10 BigEndian>~<Seq <Digit 10>~Unicode>".chars());

            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Date".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("TimeSince".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("UnixEpoch".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Duration".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seconds".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));            
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("ℕ".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("PosInt".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(10))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("BigEndian".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Digit".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(10))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
            assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
            assert_eq!( lex.next(), None );
        }
    }

    #[test]
    fn test_parse() {
        // todo
    }

    #[test]
    fn test_normalize() {
        // todo
    }
    
    #[test]
    fn test_curry() {
        // todo
    }

    #[test]
    fn test_subtype() {
        // todo
    }
}


