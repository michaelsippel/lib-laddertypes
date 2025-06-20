use {
    crate::{constraint_system::{
            subtype_unify, ConstraintError, ConstraintPair, ConstraintSystem
        }, dict::*, parser::*, term::*, Context, HashMapSubst, LayeredContext, TypeKind
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

/*
  Only Ladders
*/
#[test]
fn test_subtype_unification1() {
    let mut dict = Context::new();
    dict.add_variable("T", TypeKind::Type);

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B").unwrap(),
                rhs : dict.parse("B").unwrap()
            }
        ]).solve(),
        Ok((
            vec![ dict.parse("A").unwrap() ],
            vec![].into_iter().collect::<HashMapSubst>(),
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B ~ C ~ D").unwrap(),
                rhs : dict.parse("C ~ D").unwrap()
            }
        ]).solve(),
        Ok((
            vec![ dict.parse("A ~ B").unwrap() ],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B ~ C ~ D").unwrap(),
                rhs : dict.parse("T ~ D").unwrap()
            }
        ]).solve(),
        Ok((
            vec![ TypeTerm::unit() ],
            vec![
                (0,
                    dict.parse("A ~ B ~ C").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B ~ C ~ D").unwrap(),
                rhs : dict.parse("B ~ T ~ D").unwrap(),
            }
        ]).solve(),
        Ok((
            vec![ dict.parse("A").unwrap() ],
            vec![
                (0, dict.parse("C").unwrap())
            ].into_iter().collect()
        ))
    );
}

/*
   Variables
 */
#[test]
fn test_subtype_unification2() {
    let mut dict = Context::new();

    dict.add_variable("T", TypeKind::Type);
    dict.add_variable("U", TypeKind::Type);
    dict.add_variable("V", TypeKind::Type);
    dict.add_variable("W", TypeKind::Type);

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair{
                addr: Vec::new(),
                lhs: dict.parse("<Seq~T <Digit 10> ~ Char ~ Ascii>").unwrap(),
                rhs: dict.parse("<Seq~<LengthPrefix x86.UInt64> Char ~ Ascii>").unwrap(),
            }
        ]).solve(),
        Ok((
            vec![
                dict.parse("<Seq <Digit 10>>").unwrap()
            ],
            vec![
                // T
                (0, dict.parse("<LengthPrefix x86.UInt64>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs: dict.parse("U").unwrap(),
                rhs: dict.parse("<Seq Char>").unwrap()
            },
            ConstraintPair {
                addr : Vec::new(),
                lhs :  dict.parse("T").unwrap(),
                rhs : dict.parse("<Seq U>").unwrap(),
            }
        ]).solve(),
        Ok((
            vec![
                TypeTerm::unit(),
                TypeTerm::unit(),
            ],
            vec![
                // T
                (0, dict.parse("<Seq <Seq Char>>").unwrap()),

                // U
                (1, dict.parse("<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("<Seq T>").unwrap(),
                rhs : dict.parse("<Seq W~<Seq Char>>").unwrap(),
            },
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("<Seq~<LengthPrefix x86.UInt64> ℕ~<PosInt 10 BigEndian>>").unwrap(),
                rhs : dict.parse("<<LengthPrefix x86.UInt64> W>").unwrap()
            }
        ]).solve(),
        Ok((
            vec![
                TypeTerm::unit(),
                dict.parse("<Seq ℕ>").unwrap(),
            ],
            vec![
                // W
                (3, dict.parse("ℕ~<PosInt 10 BigEndian>").unwrap()),

                // T
                (0, dict.parse("ℕ~<PosInt 10 BigEndian>~<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        subtype_unify(
            &dict.parse("<Seq~List~Vec <Digit 16>~Char>").expect(""),
            &dict.parse("<List~Vec Char>").expect("")
        ),
        Ok((
            dict.parse("<Seq~List <Digit 16>>").expect(""),
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        subtype_unify(
            &dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq~List~Vec <Digit 16>~Char>").expect(""),
            &dict.parse("<List~Vec Char>").expect("")
        ),
        Ok((
            dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq~List <Digit 16>>").expect(""),
            vec![].into_iter().collect()
        ))
    );
}

/*
   Subtypes in some rungs
 */
#[test]
fn test_subtype_unification3() {
    let mut dict = Context::new();

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs: dict.parse("<A1~A2  B  C  D1~D2 E F1~F2>").expect("parse"),
                rhs: dict.parse("<A2 B C D2 E F2>").expect("parse")
            }
        ]).solve(),

        Ok((
            // halo
            vec![
                dict.parse("<A1~A2 B C D1~D2 E F1>").expect("parse")
            ],

            // subst
            vec![
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_sub(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs: dict.parse("<Seq~List  B  C  D1~D2 E F1~F2>").expect("parse"),
                rhs: dict.parse("<List      B  C     D2 E F2>").expect("parse")
            }
        ]).solve(),

        Ok((
            // halo
            vec![
                dict.parse("<Seq~List B C D1~D2 E F1>").expect("parse")
            ],

            // subst
            vec![
            ].into_iter().collect()
        ))
    );
}

/*
   Not a Subtype!
 */
#[test]
fn test_trait_not_subtype() {
    let mut dict = Context::new();

    assert_eq!(
        subtype_unify(
            &dict.parse("A ~ B").expect(""),
            &dict.parse("A ~ B ~ C").expect("")
        ),
        Err(ConstraintError {
            addr: vec![1],
            t1: dict.parse("B").expect(""),
            t2: dict.parse("C").expect("")
        })
    );

    assert_eq!(
        subtype_unify(
            &dict.parse("<Seq~List~Vec <Digit 10>~Char>").expect(""),
            &dict.parse("<Seq~List~Vec Char~ReprTree>").expect("")
        ),
        Err(ConstraintError {
            addr: vec![1,1],
            t1: dict.parse("Char").expect(""),
            t2: dict.parse("ReprTree").expect("")
        })
    );
}

/*
   subtype inside a sequence item
*/
#[test]
fn test_reprtree_list_subtype() {
    let mut dict = Context::new();

    dict.add_variable("Item", TypeKind::Type);

    assert_eq!(
        subtype_unify(
            &dict.parse("<List~Vec <Digit 10>~Char~ReprTree>").expect(""),
            &dict.parse("<List~Vec Item~ReprTree>").expect("")
        ),
        Ok((
            TypeTerm::unit(),
            vec![
                (0, dict.parse("<Digit 10>~Char").unwrap())
            ].into_iter().collect()
        ))
    );
}

#[test]
pub fn test_subtype_delim() {
    let mut dict = Context::new();

    dict.add_variable("T", TypeKind::Type);
    dict.add_variable("Delim", TypeKind::ValueUInt);

    assert_eq!(
        ConstraintSystem::new_sub(vec![

            ConstraintPair {
                addr: Vec::new(),
                // given type
                lhs : dict.parse("
                  < Seq <Seq <Digit 10>~Char~Ascii~UInt8> >
                ~ < ValueSep ':' Char~Ascii~UInt8 >
                ~ < Seq~<LengthPrefix UInt64> Char~Ascii~UInt8 >
                ").expect(""),

                // expected type
                rhs : dict.parse("
                  < Seq <Seq T> >
                ~ < ValueSep Delim T >
                ~ < Seq~<LengthPrefix UInt64> T >
                ").expect("")
            },

            // subtype bounds
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("T").expect(""),
                rhs : dict.parse("UInt8").expect("")
            },
            /* todo
            (
                dict.parse("<TypeOf Delim>").expect(""),
                dict.parse("T").expect("")
            ),
            */
        ]).solve(),
        Ok((
            // halo types for each rhs in the sub-equations
            vec![
                dict.parse("<Seq <Seq <Digit 10>>>").expect(""),
                dict.parse("Char~Ascii").expect(""),
            ],

            // variable substitution
            vec![
                (0, dict.parse("Char~Ascii~UInt8").expect("")),
                (1, TypeTerm::Char(':')),
            ].into_iter().collect()
        ))
    );
}
