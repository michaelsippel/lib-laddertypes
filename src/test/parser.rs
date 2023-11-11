
use {
    crate::{term::*, dict::*, parser::*}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_parser_id() {
    let mut dict = TypeDict::new();

    dict.add_varname("T".into());

    assert_eq!(
        Ok(TypeTerm::TypeID(TypeID::Var(0))),
        dict.parse("T")
    );

    assert_eq!(
        Ok(TypeTerm::TypeID(TypeID::Fun(0))),
        dict.parse("A")
    );
}

#[test]
fn test_parser_num() {
    assert_eq!(
        Ok(TypeTerm::Num(1234)),
        TypeDict::new().parse("1234")
    );
}

#[test]
fn test_parser_char() {
    assert_eq!(
        Ok(TypeTerm::Char('x')),
        TypeDict::new().parse("'x'")
    );
}

#[test]
fn test_parser_app() {
    assert_eq!(
        TypeDict::new().parse("<A B>"),
        Ok(TypeTerm::App(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::TypeID(TypeID::Fun(1)),
        ]))
    );
    assert_eq!(
        TypeDict::new().parse("<A B C>"),
        Ok(TypeTerm::App(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::TypeID(TypeID::Fun(1)),
            TypeTerm::TypeID(TypeID::Fun(2)),
        ]))
    );
}

#[test]
fn test_parser_unexpected_close() {
    assert_eq!(
        TypeDict::new().parse(">"),
        Err(ParseError::UnexpectedClose)
    );
}

#[test]
fn test_parser_unexpected_token() {
    assert_eq!(
        TypeDict::new().parse("A B"),
        Err(ParseError::UnexpectedToken)
    );
}

#[test]
fn test_parser_ladder() {
    assert_eq!(
        TypeDict::new().parse("A~B"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::TypeID(TypeID::Fun(1)),
        ]))
    );
    assert_eq!(
        TypeDict::new().parse("A~B~C"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::TypeID(TypeID::Fun(1)),
            TypeTerm::TypeID(TypeID::Fun(2)),
        ]))
    );
}

#[test]
fn test_parser_ladder_outside() {
    assert_eq!(
        TypeDict::new().parse("<A B>~C"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::App(vec![
                TypeTerm::TypeID(TypeID::Fun(0)),
                TypeTerm::TypeID(TypeID::Fun(1)),
            ]),
            TypeTerm::TypeID(TypeID::Fun(2)),
        ]))
    );    
}

#[test]
fn test_parser_ladder_inside() {
    assert_eq!(
        TypeDict::new().parse("<A B~C>"),
        Ok(TypeTerm::App(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::Ladder(vec![
                TypeTerm::TypeID(TypeID::Fun(1)),
                TypeTerm::TypeID(TypeID::Fun(2)),
            ])
        ]))
    );    
}

#[test]
fn test_parser_ladder_between() {
    assert_eq!(
        TypeDict::new().parse("<A B~<C D>>"),
        Ok(TypeTerm::App(vec![
            TypeTerm::TypeID(TypeID::Fun(0)),
            TypeTerm::Ladder(vec![
                TypeTerm::TypeID(TypeID::Fun(1)),
                TypeTerm::App(vec![
                    TypeTerm::TypeID(TypeID::Fun(2)),
                    TypeTerm::TypeID(TypeID::Fun(3)),
                ])
            ])
        ]))
    );    
}


#[test]
fn test_parser_ladder_large() {
    assert_eq!(
        TypeDict::new().parse(
            "<Seq Date
                  ~<TimeSince UnixEpoch>
                  ~<Duration Seconds>
                  ~ℕ
                  ~<PosInt 10 BigEndian>
                  ~< Seq <Digit 10>~Unicode > >
              ~<SepSeq Unicode ':'>
              ~<Seq Unicode>
              ~UTF-8
              ~<Seq Byte>"),

        Ok(
            TypeTerm::Ladder(vec![
                TypeTerm::App(vec![
                    TypeTerm::TypeID(TypeID::Fun(0)),
                    TypeTerm::Ladder(vec![
                        TypeTerm::TypeID(TypeID::Fun(1)),
                        TypeTerm::App(vec![
                            TypeTerm::TypeID(TypeID::Fun(2)),
                            TypeTerm::TypeID(TypeID::Fun(3))
                        ]),
                        TypeTerm::App(vec![
                            TypeTerm::TypeID(TypeID::Fun(4)),
                            TypeTerm::TypeID(TypeID::Fun(5))
                        ]),
                        TypeTerm::TypeID(TypeID::Fun(6)),
                        TypeTerm::App(vec![
                            TypeTerm::TypeID(TypeID::Fun(7)),
                            TypeTerm::Num(10),
                            TypeTerm::TypeID(TypeID::Fun(8))
                        ]),
                        TypeTerm::App(vec![
                            TypeTerm::TypeID(TypeID::Fun(0)),
                            TypeTerm::Ladder(vec![
                                TypeTerm::App(vec![
                                    TypeTerm::TypeID(TypeID::Fun(9)),
                                    TypeTerm::Num(10)
                                ]),
                                TypeTerm::TypeID(TypeID::Fun(10))
                            ])
                        ])
                    ])
                ]),
                TypeTerm::App(vec![
                    TypeTerm::TypeID(TypeID::Fun(11)),
                    TypeTerm::TypeID(TypeID::Fun(10)),
                    TypeTerm::Char(':')
                ]),
                TypeTerm::App(vec![
                    TypeTerm::TypeID(TypeID::Fun(0)),
                    TypeTerm::TypeID(TypeID::Fun(10))
                ]),
                TypeTerm::TypeID(TypeID::Fun(12)),
                TypeTerm::App(vec![
                    TypeTerm::TypeID(TypeID::Fun(0)),
                    TypeTerm::TypeID(TypeID::Fun(13))
                ])
            ])
        )
    );
}

