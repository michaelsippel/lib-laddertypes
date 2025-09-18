/*
   lib-laddertypes
   Copyright (C) 2023-2025  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use {
    crate::{dict::*, parser::*, ConstraintPair, Context, ContextEntry, EnumVariant, LayeredContext, StructMember, TypeKind, TypeTerm, VariableConstraint},
    tiny_diagnostics::InputRegionTag
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
        Err((InputRegionTag{ begin:0, end:0 }, ParseError::UnexpectedClose))
    );
}

#[test]
fn test_parser_unexpected_token() {
    assert_eq!(
        Context::new().parse("A B"),
        Err((InputRegionTag{ begin:2, end:3 }, ParseError::UnexpectedToken))
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
            item: Box::new(TypeTerm::Id(0))
        })
    );
}

#[test]
fn test_parser_seq_repr() {
    assert_eq!(
        Context::new().parse("[~A B]"),
        Ok(TypeTerm::Seq{
            seq_repr: Some(Box::new(TypeTerm::Id(0))),
            item: Box::new(TypeTerm::Id(1))
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
fn test_parser_func() {
    assert_eq!(
        Context::new().parse("A --> B"),
        Ok(TypeTerm::Func(vec![
            TypeTerm::Id(0),
            TypeTerm::Id(1)
        ]))
    );
    assert_eq!(
        Context::new().parse("A~X --> B~Y"),
        Ok(TypeTerm::Func(vec![
            TypeTerm::Ladder(vec![ TypeTerm::Id(0), TypeTerm::Id(1) ]),
            TypeTerm::Ladder(vec![ TypeTerm::Id(2), TypeTerm::Id(3) ])
        ]))
    );
}

#[test]
fn test_parser_morph() {
    assert_eq!(
        Context::new().parse("A -morph-> B"),
        Ok(TypeTerm::Morph(
            Box::new(TypeTerm::Id(0)),
            Box::new(TypeTerm::Id(1))
        ))
    );
    assert_eq!(
        Context::new().parse("A~X -morph-> B~Y"),
        Ok(TypeTerm::Morph(
            Box::new(TypeTerm::Ladder(vec![ TypeTerm::Id(0), TypeTerm::Id(1) ])),
            Box::new(TypeTerm::Ladder(vec![ TypeTerm::Id(2), TypeTerm::Id(3) ]))
        ))
    );
}

#[test]
fn test_parser_univ() {
    assert_eq!(
        Context::new().parse("∀T (T:<=native.UInt8) [~A T]"),
        Ok(TypeTerm::Univ{
            Γ: vec![
                ContextEntry{ symbol: "T".into(), kind: TypeKind::Type }
            ],
            bounds: vec![
                ConstraintPair::Subtype(TypeTerm::Var(0), TypeTerm::Id(0))
            ],
            τ: Box::new(TypeTerm::Seq {
                seq_repr: Some(Box::new(TypeTerm::Id(1))),
                item: Box::new(TypeTerm::Var(0))
            })
        })
    );
}

#[test]
fn test_parser_univ2() {
    assert_eq!(
        Context::new().parse("∀T ∀U (T:<=native.UInt8) (U:<=native.UInt16) [~A T]"),

        Ok(TypeTerm::Univ {
            Γ: vec![
                ContextEntry{ symbol: "T".into(), kind: TypeKind::Type },
                ContextEntry{ symbol: "U".into(), kind: TypeKind::Type },
            ],
            bounds: vec![
                ConstraintPair::Subtype(TypeTerm::Var(0), TypeTerm::Id(0)),
                ConstraintPair::Subtype(TypeTerm::Var(1), TypeTerm::Id(1)),
            ],
            τ: Box::new(TypeTerm::Seq {
                seq_repr: Some(Box::new(TypeTerm::Id(2))),
                item: Box::new(TypeTerm::Var(0))
            })
        })
    );
}

#[test]
fn test_parser_univ_val1() {
    assert_eq!(
        Context::new().parse("∀X:ℕ A"),
        Ok(TypeTerm::Univ{
            Γ: vec![
                ContextEntry{ symbol: "X".into(), kind: TypeKind::Value(TypeTerm::Id(0)) }
            ],
            bounds: Vec::new(),
            τ: Box::new(TypeTerm::Id(1))
        })
    );
}

#[test]
fn test_parser_univ_val() {
    assert_eq!(
        Context::new().parse("∀Len:ℕ [~<array.Static Len> Char]"),
        Ok(TypeTerm::Univ{
            Γ: vec![
                ContextEntry{ symbol: "Len".into(), kind: TypeKind::Value(TypeTerm::Id(0)) }
            ],
            bounds: Vec::new(),
            τ: Box::new(TypeTerm::Seq {
                seq_repr: Some(Box::new(TypeTerm::Spec(vec![TypeTerm::Id(1), TypeTerm::Var(0)]))),
                item: Box::new(TypeTerm::Id(2))
            })
        })
    );
}

#[test]
fn test_parser_univ3() {
    assert_eq!(
        Context::new().parse("
                ∀T  ∀End:T  ∀LenType (LenType :>< ℕ)
                         [~<array.ValueTerminated End> T]
                -morph-> [~<array.LengthPrefix LenType> T]
        "),

        Ok(
            TypeTerm::Univ {
                Γ: vec![
                    ContextEntry{ symbol: "T".into(), kind: TypeKind::Type },
                    ContextEntry{ symbol: "End".into(), kind: TypeKind::Value(TypeTerm::Var(0)) },
                    ContextEntry{ symbol: "LenType".into(), kind: TypeKind::Type }
                ],
                bounds: vec![
                    ConstraintPair::Trait(TypeTerm::Var(2), TypeTerm::Id(0))
                ],
                τ: Box::new(TypeTerm::Morph(
                    Box::new(TypeTerm::Seq{ seq_repr: Some(Box::new(TypeTerm::Spec(vec![ TypeTerm::Id(1), TypeTerm::Var(1) ]))), item: Box::new(TypeTerm::Var(0)) }),
                    Box::new(TypeTerm::Seq{ seq_repr: Some(Box::new(TypeTerm::Spec(vec![ TypeTerm::Id(2), TypeTerm::Var(2) ]))), item: Box::new(TypeTerm::Var(0)) })
                ))
            }
        )
    );
}

#[test]
fn test_parser_univ_morph() {
    assert_eq!(
        Context::new().parse("∀T (T:<=X) [~A T] -morph-> [~B T]"),
        Ok(TypeTerm::Univ{
            Γ : vec![
                ContextEntry{ symbol:"T".into(), kind: TypeKind::Type }
            ],
            bounds: vec![
                ConstraintPair::Subtype(TypeTerm::Var(0), TypeTerm::Id(0))
            ],
            τ: Box::new(
                TypeTerm::Morph(
                    Box::new(TypeTerm::Seq { seq_repr: Some(Box::new(TypeTerm::Id(1))), item: Box::new(TypeTerm::Var(0)) }),
                    Box::new(TypeTerm::Seq { seq_repr: Some(Box::new(TypeTerm::Id(2))), item: Box::new(TypeTerm::Var(0)) }),
                )
            )
        })
    );
}
