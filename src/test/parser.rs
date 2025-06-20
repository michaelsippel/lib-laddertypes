
use {
    crate::{dict::*, parser::*, Context, EnumVariant, LayeredContext, StructMember, TypeKind, TypeTerm, VariableConstraint}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_parser_id() {
    let mut dict = Context::new();

    dict.add_variable("T", TypeKind::Type);

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
        Context::new().parse("1234")
    );
}

#[test]
fn test_parser_char() {
    assert_eq!(
        Ok(TypeTerm::Char('x')),
        Context::new().parse("'x'")
    );
}

#[test]
fn test_parser_app() {
    assert_eq!(
        Context::new().parse("<A B>"),
        Ok(TypeTerm::Spec(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
        ]))
    );
    assert_eq!(
        Context::new().parse("<A B C>"),
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
        Context::new().parse(">"),
        Err(ParseError::UnexpectedClose)
    );
}

#[test]
fn test_parser_unexpected_token() {
    assert_eq!(
        Context::new().parse("A B"),
        Err(ParseError::UnexpectedToken)
    );
}

#[test]
fn test_parser_ladder() {
    assert_eq!(
        Context::new().parse("A~B"),
        Ok(TypeTerm::Ladder(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1),
        ]))
    );
    assert_eq!(
        Context::new().parse("A~B~C"),
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
        Context::new().parse("<A B>~C"),
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
        Context::new().parse("<A B~C>"),
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
        Context::new().parse("<A B~<C D>>"),
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
        Context::new().parse(
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
                TypeTerm::Spec(vec![
                    TypeTerm::Id(0),
                    TypeTerm::Ladder(vec![
                        TypeTerm::Id(1),
                        TypeTerm::Spec(vec![
                            TypeTerm::Id(2),
                            TypeTerm::Id(3)
                        ]),
                        TypeTerm::Spec(vec![
                            TypeTerm::Id(4),
                            TypeTerm::Id(5)
                        ]),
                        TypeTerm::Id(6),
                        TypeTerm::Spec(vec![
                            TypeTerm::Id(7),
                            TypeTerm::Num(10),
                            TypeTerm::Id(8)
                        ]),
                        TypeTerm::Spec(vec![
                            TypeTerm::Id(0),
                            TypeTerm::Ladder(vec![
                                TypeTerm::Spec(vec![
                                    TypeTerm::Id(9),
                                    TypeTerm::Num(10)
                                ]),
                                TypeTerm::Id(10)
                            ])
                        ])
                    ])
                ]),
                TypeTerm::Spec(vec![
                    TypeTerm::Id(11),
                    TypeTerm::Id(10),
                    TypeTerm::Char(':')
                ]),
                TypeTerm::Spec(vec![
                    TypeTerm::Id(0),
                    TypeTerm::Id(10)
                ]),
            TypeTerm::Id(12),
                TypeTerm::Spec(vec![
                    TypeTerm::Id(0),
                    TypeTerm::Id(13)
                ])
            ])
        )
    );
}



#[test]
fn test_parser_seq() {
    assert_eq!(
        Context::new().parse("[A]"),
        Ok(TypeTerm::Seq{
            seq_repr: None,
            items: vec![ TypeTerm::Id(0) ]
        })
    );
}

#[test]
fn test_parser_seq_repr() {
    assert_eq!(
        Context::new().parse("[~A B]"),
        Ok(TypeTerm::Seq{
            seq_repr: Some(Box::new(TypeTerm::Id(0))),
            items: vec![ TypeTerm::Id(1) ]
        })
    );
}

#[test]
fn test_parser_struct() {
    assert_eq!(
        Context::new().parse("{ a: A; b: B; }"),
        Ok(TypeTerm::Struct{
            struct_repr: None,
            members: vec![
                StructMember{ symbol: "a".into(), ty: TypeTerm::Id(0) },
                StructMember{ symbol: "b".into(), ty: TypeTerm::Id(1) },
            ]
        })
    );
}
#[test]
fn test_parser_struct_repr() {
    assert_eq!(
        Context::new().parse("{~X a: A; b: B; }"),
        Ok(TypeTerm::Struct{
            struct_repr: Some(Box::new(TypeTerm::Id(0))),
            members: vec![
                StructMember{ symbol: "a".into(), ty: TypeTerm::Id(1) },
                StructMember{ symbol: "b".into(), ty: TypeTerm::Id(2) },
            ]
        })
    );
}

#[test]
fn test_parser_enum() {
    assert_eq!(
        Context::new().parse("{ | a: A | b: B }"),
        Ok(TypeTerm::Enum{
            enum_repr: None,
            variants: vec![
                EnumVariant{ symbol: "a".into(), ty: TypeTerm::Id(0) },
                EnumVariant{ symbol: "b".into(), ty: TypeTerm::Id(1) },
            ]
        })
    );
}

#[test]
fn test_parser_univ_val() {
    assert_eq!(
        Context::new().parse("∀(Len:ℕ) [~<array.Static Len> Char]"),
        Ok(TypeTerm::Univ(
            Box::new(VariableConstraint::ValueUInt),
            Box::new(TypeTerm::Seq { seq_repr: Some(Box::new(TypeTerm::Spec(vec![TypeTerm::Id(1), TypeTerm::Var(0)]))), items: vec![ TypeTerm::Id(2) ] })
        ))
    );
}

#[test]
fn test_parser_univ() {
    assert_eq!(
        Context::new().parse("∀(T:<=native.UInt8) [~A T]"),
        Ok(TypeTerm::Univ(
            Box::new(VariableConstraint::Subtype(TypeTerm::Id(0))),
            Box::new(TypeTerm::Seq { seq_repr: Some(Box::new(TypeTerm::Id(1))), items: vec![ TypeTerm::Var(0) ] })
        ))
    );
}

#[test]
fn test_parser_univ2() {
    assert_eq!(
        Context::new().parse("∀(T:<=native.UInt8) ∀(U:<=native.UInt16) [~A T]"),
        Ok(TypeTerm::Univ(
            Box::new(VariableConstraint::Subtype(TypeTerm::Id(0))),

            Box::new(
                TypeTerm::Univ(
                    Box::new(VariableConstraint::Subtype(TypeTerm::Id(1))),
                    Box::new(TypeTerm::Seq { seq_repr: Some(Box::new(TypeTerm::Id(2))), items: vec![ TypeTerm::Var(1) ] })
                )
            )
        ))
    );
}
