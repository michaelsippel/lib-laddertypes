
use {
    crate::{dict::*, parser::*, constraint_system::ConstraintError},
    std::iter::FromIterator
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

fn test_unify(ts1: &str, ts2: &str, expect_unificator: bool) {
    let mut dict = BimapTypeDict::new();
    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    let mut t1 = dict.parse(ts1).unwrap();
    let mut t2 = dict.parse(ts2).unwrap();
    let σ = crate::unify( &t1, &t2 );

    if expect_unificator {
        assert!(σ.is_ok());

        let σ = σ.unwrap();

        assert_eq!(
            t1.apply_subst(&σ),
            t2.apply_subst(&σ)
        );
    } else {
        assert!(! σ.is_ok());
    }
}

#[test]
fn test_unification_error() {
    let mut dict = BimapTypeDict::new();
    dict.add_varname(String::from("T"));

    assert_eq!(
        crate::unify(
            &dict.parse("<A T>").unwrap(),
            &dict.parse("<B T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![0],
            t1: dict.parse("A").unwrap(),
            t2: dict.parse("B").unwrap()
        })
    );

    assert_eq!(
        crate::unify(
            &dict.parse("<V <U A> T>").unwrap(),
            &dict.parse("<V <U B> T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![1, 1],
            t1: dict.parse("A").unwrap(),
            t2: dict.parse("B").unwrap()
        })
    );

    assert_eq!(
        crate::unify(
            &dict.parse("T").unwrap(),
            &dict.parse("<Seq T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![],
            t1: dict.parse("T").unwrap(),
            t2: dict.parse("<Seq T>").unwrap()
        })
    );
}

#[test]
fn test_unification() {
    test_unify("A", "A", true);
    test_unify("A", "B", false);
    test_unify("<Seq T>", "<Seq Ascii~Char>", true);

    // this worked easily with desugared terms,
    // but is a weird edge case with sugared terms
    // not relevant now
    //test_unify("<Seq T>", "<U Char>", true);

    test_unify(
        "<Seq Path~<Seq Char>>~<SepSeq Char '\\n'>~<Seq Char>",
        "<Seq T~<Seq Char>>~<SepSeq Char '\\n'>~<Seq Char>",
        true
    );

    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    assert_eq!(
        ConstraintSystem::new_eq(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs: dict.parse("U").unwrap(),
                rhs: dict.parse("<Seq Char>").unwrap()
            },
            ConstraintPair {
                addr: Vec::new(),
                lhs: dict.parse("T").unwrap(),
                rhs: dict.parse("<Seq U>").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![
                // T
                (TypeID::Var(0), dict.parse("<Seq <Seq Char>>").unwrap()),

                // U
                (TypeID::Var(1), dict.parse("<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_eq(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("<Seq T>").unwrap(),
                rhs : dict.parse("<Seq W~<Seq Char>>").unwrap()
            },
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("<Seq ℕ>").unwrap(),
                rhs : dict.parse("<Seq W>").unwrap(),
            }
        ]).solve(),
        Ok((
            vec![],
            vec![
                // W
                (TypeID::Var(3), dict.parse("ℕ").unwrap()),

                // T
                (TypeID::Var(0), dict.parse("ℕ~<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );
}

#[test]
fn test_subtype_unification1() {
    let mut dict = BimapTypeDict::new();
    dict.add_varname(String::from("T"));

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
            vec![].into_iter().collect()
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
                (dict.get_typeid(&"T".into()).unwrap(),
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
                (dict.get_typeid(&"T".into()).unwrap(), dict.parse("C").unwrap())
            ].into_iter().collect()
        ))
    );
}

#[test]
fn test_subtype_unification2() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

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
                (TypeID::Var(0), dict.parse("<LengthPrefix x86.UInt64>").unwrap())
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
                (TypeID::Var(0), dict.parse("<Seq <Seq Char>>").unwrap()),

                // U
                (TypeID::Var(1), dict.parse("<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );

    eprintln!("=&==========&======&=====&=====&==");

    if let Ok((ψ,σ)) =
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
        ]).solve() {
            for ( k,v) in σ.iter() {
                eprintln!(" {:?} ==> {} ",
                    dict.get_typename(&k),
                    v.pretty(&dict, 0));
            }

        assert_eq!(ψ,
            vec![
                TypeTerm::unit(),
                dict.parse("<Seq ℕ>").unwrap(),
            ]);
        assert_eq!(σ,
            vec![
                // W
                (TypeID::Var(3), dict.parse("ℕ~<PosInt 10 BigEndian>").unwrap()),

                // T
                (TypeID::Var(0), dict.parse("ℕ~<PosInt 10 BigEndian>~<Seq Char>").unwrap())
            ].into_iter().collect()
        );
    } else {
        assert!(false);
    }

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

#[test]
fn test_subtype_unification3() {
    let mut dict = BimapTypeDict::new();

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


#[test]
fn test_trait_not_subtype() {
    let mut dict = BimapTypeDict::new();

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
            addr: vec![1],
            t1: dict.parse("Char").expect(""),
            t2: dict.parse("ReprTree").expect("")
        })
    );
}

#[test]
fn test_reprtree_list_subtype() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname("Item".into());

    assert_eq!(
        subtype_unify(
            &dict.parse("<List~Vec <Digit 10>~Char~ReprTree>").expect(""),
            &dict.parse("<List~Vec Item~ReprTree>").expect("")
        ),
        Ok((
            TypeTerm::unit(),
            vec![
                (dict.get_typeid(&"Item".into()).unwrap(), dict.parse("<Digit 10>~Char").unwrap())
            ].into_iter().collect()
        ))
    );
}

#[test]
pub fn test_subtype_delim() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("Delim"));

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
                (dict.get_typeid(&"T".into()).unwrap(), dict.parse("Char~Ascii~UInt8").expect("")),
                (dict.get_typeid(&"Delim".into()).unwrap(), TypeTerm::Char(':')),
            ].into_iter().collect()
        ))
    );
}



use crate::{subtype_unify, term::*, constraint_system::{ConstraintPair, ConstraintSystem}};

#[test]
fn test_list_subtype_sugared() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname("Item".into());

    let subtype_constraints = vec![
        ConstraintPair::new(
            dict.parse("<List~Vec <Digit 10>~Char~ReprTree>").expect(""),
            dict.parse("<List~Vec Item~ReprTree>").expect("")
        )
    ];

    assert_eq!(
        ConstraintSystem::new_sub(subtype_constraints).solve(),
        Ok((
            vec![ TypeTerm::Ladder(vec![]) ],
            vec![
                (dict.get_typeid(&"Item".into()).unwrap(),
                    dict.parse("<Digit 10>~Char").unwrap())
            ].into_iter().collect()
        ))
    );
}

#[test]
pub fn test_subtype_delim_sugared() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("Delim"));

    let subtype_constraints = vec![
        ConstraintPair::new(
            dict.parse("
              < Seq <Seq <Digit 10>~Char~Ascii~UInt8> >
            ~ < ValueSep ':' Char~Ascii~UInt8 >
            ~ < Seq~<LengthPrefix UInt64> Char~Ascii~UInt8 >
            ").expect(""),

            dict.parse("
              < Seq <Seq T> >
            ~ < ValueSep Delim T >
            ~ < Seq~<LengthPrefix UInt64> T >
            ").expect(""),
        ),
        ConstraintPair::new(
            dict.parse("T").expect(""),
            dict.parse("UInt8").expect("")
        ),
    ];

    assert_eq!(
        ConstraintSystem::new_sub(subtype_constraints).solve(),
        Ok((
            // halo types for each rhs in the sub-equations
            vec![
                dict.parse("<Seq <Seq <Digit 10>>>").expect(""),
                dict.parse("Char~Ascii").expect(""),
            ],

            // variable substitution
            vec![
                (dict.get_typeid(&"T".into()).unwrap(), dict.parse("Char~Ascii~UInt8").expect("")),
                (dict.get_typeid(&"Delim".into()).unwrap(), TypeTerm::Char(':')),
            ].into_iter().collect()
        ))
    );
}
