use {
    crate::{dict::*, parser::*,
        constraint_system::{
            ConstraintSystem,
            ConstraintPair,
            ConstraintError
        }
    }
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

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
