use crate::lexer::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_lexer_symbol() {
    let mut lex = LadderTypeLexer::from("symbol".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("symbol".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num() {
    let mut lex = LadderTypeLexer::from("1234".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(1234))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num_error() {
    let mut lex = LadderTypeLexer::from("123xxx".chars());
    assert_eq!( lex.next(), Some(Err(LexError::InvalidDigit)) );
}

#[test]
fn test_lexer_char() {
    let mut lex = LadderTypeLexer::from("'x'".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Char('x'))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_char_error() {
    let mut lex = LadderTypeLexer::from("'xx'".chars());
    assert_eq!( lex.next(), Some(Err(LexError::InvalidChar)) );
}

#[test]
fn test_lexer_ladder() {
    let mut lex = LadderTypeLexer::from("abc~def".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_ladder_space() {
    let mut lex = LadderTypeLexer::from("abc   ~ def".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app() {
    let mut lex = LadderTypeLexer::from("<Seq Char>".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app_space() {
    let mut lex = LadderTypeLexer::from("   <Seq      Char  >".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_large() {
    let mut lex = LadderTypeLexer::from(
        "<Seq Date
              ~<TimeSince UnixEpoch>
              ~<Duration Seconds>
              ~ℕ
              ~<PosInt 10 BigEndian>
              ~< Seq <Digit 10>~Unicode > >
         ~<SepSeq Unicode ':'>
         ~<Seq Unicode>
         ~UTF-8
         ~<Seq Byte>".chars());

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
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("SepSeq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Char(':'))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("UTF-8".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Byte".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)));

    assert_eq!( lex.next(), None );
}

