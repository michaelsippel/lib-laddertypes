
use {
    crate::{desugared_term::*, dict::*, parser::*, Context, LayeredContext, TypeKind, TypeTerm}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_parser_id() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname("T".into());

    assert_eq!(
        Ok(TypeTerm::Var(0)),
        dict.parse("T")
    );

    assert_eq!(
        Ok(TypeTerm::Id(0)),
        dict.parse("A")
    );
}

#[test]
fn test_parser_var_ctx() {
    let mut ctx = Context::new();

    ctx.add_variable("T", TypeKind::Type);

    assert_eq!(
        Ok(TypeTerm::Var(0)),
        ctx.parse("T")
    );

    assert_eq!(
        Ok(TypeTerm::Id(0)),
        ctx.parse("A")
    );
}

#[test]
fn test_parser_num() {
    assert_eq!(
        Ok(TypeTerm::Num(1234)),
        BimapTypeDict::new().parse("1234")
    );
}

#[test]
fn test_parser_char() {
    assert_eq!(
        Ok(TypeTerm::Char('x')),
        BimapTypeDict::new().parse("'x'")
    );
}

#[test]
fn test_parser_app() {
    assert_eq!(
        BimapTypeDict::new().parse("<A B>"),
        Ok(TypeTerm::Spec(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
        ]))
    );
    assert_eq!(
        BimapTypeDict::new().parse("<A B C>"),
        Ok(TypeTerm::Spec(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
            TypeTerm::Id(2),
        ]))
    );
}

#[test]
fn test_parser_unexpected_close() {
    assert_eq!(
        BimapTypeDict::new().parse(">"),
        Err(ParseError::UnexpectedClose)
    );
}

#[test]
fn test_parser_unexpected_token() {
    assert_eq!(
        BimapTypeDict::new().parse("A B"),
        Err(ParseError::UnexpectedToken)
    );
}

#[test]
fn test_parser_ladder() {
    assert_eq!(
        BimapTypeDict::new().parse("A~B"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
        ]))
    );
    assert_eq!(
        BimapTypeDict::new().parse("A~B~C"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
            TypeTerm::Id(2),
        ]))
    );
}

#[test]
fn test_parser_ladder_outside() {
    assert_eq!(
        BimapTypeDict::new().parse("<A B>~C"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::Spec(vec![
                TypeTerm::Id(0),
                TypeTerm::Id(1),
            ]),
            TypeTerm::Id(2),
        ]))
    );
}

#[test]
fn test_parser_ladder_inside() {
    assert_eq!(
        BimapTypeDict::new().parse("<A B~C>"),
        Ok(TypeTerm::Spec(vec![
            TypeTerm::Id(0),
            TypeTerm::Ladder(vec![
                TypeTerm::Id(1),
                TypeTerm::Id(2),
            ])
        ]))
    );
}

#[test]
fn test_parser_ladder_between() {
    assert_eq!(
        BimapTypeDict::new().parse("<A B~<C D>>"),
        Ok(TypeTerm::Spec(vec![
            TypeTerm::Id(0),
            TypeTerm::Ladder(vec![
                TypeTerm::Id(1),
                TypeTerm::Spec(vec![
                    TypeTerm::Id(2),
                    TypeTerm::Id(3),
                ])
            ])
        ]))
    );
}


#[test]
fn test_parser_ladder_large() {
    assert_eq!(
        BimapTypeDict::new().parse_desugared(
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
            DesugaredTypeTerm::Ladder(vec![
                DesugaredTypeTerm::App(vec![
                    DesugaredTypeTerm::TypeID(TypeID::Fun(0)),
                    DesugaredTypeTerm::Ladder(vec![
                        DesugaredTypeTerm::TypeID(TypeID::Fun(1)),
                        DesugaredTypeTerm::App(vec![
                            DesugaredTypeTerm::TypeID(TypeID::Fun(2)),
                            DesugaredTypeTerm::TypeID(TypeID::Fun(3))
                        ]),
                        DesugaredTypeTerm::App(vec![
                            DesugaredTypeTerm::TypeID(TypeID::Fun(4)),
                            DesugaredTypeTerm::TypeID(TypeID::Fun(5))
                        ]),
                        DesugaredTypeTerm::TypeID(TypeID::Fun(6)),
                        DesugaredTypeTerm::App(vec![
                            DesugaredTypeTerm::TypeID(TypeID::Fun(7)),
                            DesugaredTypeTerm::Num(10),
                            DesugaredTypeTerm::TypeID(TypeID::Fun(8))
                        ]),
                        DesugaredTypeTerm::App(vec![
                            DesugaredTypeTerm::TypeID(TypeID::Fun(0)),
                            DesugaredTypeTerm::Ladder(vec![
                                DesugaredTypeTerm::App(vec![
                                    DesugaredTypeTerm::TypeID(TypeID::Fun(9)),
                                    DesugaredTypeTerm::Num(10)
                                ]),
                                DesugaredTypeTerm::TypeID(TypeID::Fun(10))
                            ])
                        ])
                    ])
                ]),
                DesugaredTypeTerm::App(vec![
                    DesugaredTypeTerm::TypeID(TypeID::Fun(11)),
                    DesugaredTypeTerm::TypeID(TypeID::Fun(10)),
                    DesugaredTypeTerm::Char(':')
                ]),
                DesugaredTypeTerm::App(vec![
                    DesugaredTypeTerm::TypeID(TypeID::Fun(0)),
                    DesugaredTypeTerm::TypeID(TypeID::Fun(10))
                ]),
                DesugaredTypeTerm::TypeID(TypeID::Fun(12)),
                DesugaredTypeTerm::App(vec![
                    DesugaredTypeTerm::TypeID(TypeID::Fun(0)),
                    DesugaredTypeTerm::TypeID(TypeID::Fun(13))
                ])
            ])
        )
    );
}
