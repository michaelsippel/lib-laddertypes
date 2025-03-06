
use {
    crate::{dict::*, parser::*, unparser::*, term::*, unification::*},
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
            t1.apply_substitution(&|v| σ.get(v).cloned()),
            t2.apply_substitution(&|v| σ.get(v).cloned())
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

        Err(UnificationError {
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

        Err(UnificationError {
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

        Err(UnificationError {
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
    test_unify("<Seq T>", "<U Char>", true);

    test_unify(
        "<Seq Path~<Seq Char>>~<SepSeq Char '\n'>~<Seq Char>",
        "<Seq T~<Seq Char>>~<SepSeq Char '\n'>~<Seq Char>",
        true
    );

    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    assert_eq!(
        UnificationProblem::new_eq(vec![
            (dict.parse("U").unwrap(), dict.parse("<Seq Char>").unwrap()),
            (dict.parse("T").unwrap(), dict.parse("<Seq U>").unwrap()),
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
        UnificationProblem::new_eq(vec![
            (dict.parse("<Seq T>").unwrap(), dict.parse("<Seq W~<Seq Char>>").unwrap()),
            (dict.parse("<Seq ℕ>").unwrap(), dict.parse("<Seq W>").unwrap()),
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
fn test_subtype_unification() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    assert_eq!(
        UnificationProblem::new_sub(vec![
            (dict.parse("<Seq~T <Digit 10> ~ Char>").unwrap(),
                dict.parse("<Seq~<LengthPrefix x86.UInt64> Char ~ Ascii>").unwrap()),
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
        UnificationProblem::new_sub(vec![
            (dict.parse("U").unwrap(), dict.parse("<Seq Char>").unwrap()),
            (dict.parse("T").unwrap(), dict.parse("<Seq U>").unwrap()),
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

    assert_eq!(
        UnificationProblem::new_sub(vec![
            (dict.parse("<Seq T>").unwrap(),
                dict.parse("<Seq W~<Seq Char>>").unwrap()),
            (dict.parse("<Seq~<LengthPrefix x86.UInt64> ℕ~<PosInt 10 BigEndian>>").unwrap(),
                dict.parse("<<LengthPrefix x86.UInt64> W>").unwrap()),
        ]).solve(),
        Ok((
            vec![
                TypeTerm::unit(),
                dict.parse("<Seq ℕ>").unwrap(),
            ],
            vec![
                // W
                (TypeID::Var(3), dict.parse("ℕ~<PosInt 10 BigEndian>").unwrap()),

                // T
                (TypeID::Var(0), dict.parse("ℕ~<PosInt 10 BigEndian>~<Seq Char>").unwrap())
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


#[test]
pub fn test_subtype_delim() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("Delim"));

    assert_eq!(
        UnificationProblem::new_sub(vec![

            (
                //given type
                dict.parse("
                  < Seq <Seq <Digit 10>~Char~Ascii~UInt8> >
                ~ < ValueSep ':' Char~Ascii~UInt8 >
                ~ < Seq~<LengthPrefix UInt64> Char~Ascii~UInt8 >
                ").expect(""),

                //expected type
                dict.parse("
                  < Seq <Seq T> >
                ~ < ValueSep Delim T >
                ~ < Seq~<LengthPrefix UInt64> T >
                ").expect("")
            ),

            // subtype bounds
            (
                dict.parse("T").expect(""),
                dict.parse("UInt8").expect("")
            ),
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
