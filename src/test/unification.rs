
use {
    crate::{dict::*, term::*, unification::*},
    std::iter::FromIterator
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

fn test_unify(ts1: &str, ts2: &str, expect_unificator: bool) {
    let mut dict = TypeDict::new();
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
    let mut dict = TypeDict::new();
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

    let mut dict = TypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    assert_eq!(
        UnificationProblem::new(vec![
            (dict.parse("U").unwrap(), dict.parse("<Seq Char>").unwrap()),
            (dict.parse("T").unwrap(), dict.parse("<Seq U>").unwrap()),
        ]).solve(),
        Ok(
            vec![
                // T
                (TypeID::Var(0), dict.parse("<Seq <Seq Char>>").unwrap()),

                // U
                (TypeID::Var(1), dict.parse("<Seq Char>").unwrap())
            ].into_iter().collect()
        )
    );

    assert_eq!(
        UnificationProblem::new(vec![
            (dict.parse("<Seq T>").unwrap(), dict.parse("<Seq W~<Seq Char>>").unwrap()),
            (dict.parse("<Seq ℕ>").unwrap(), dict.parse("<Seq W>").unwrap()),
        ]).solve(),
        Ok(
            vec![
                // W
                (TypeID::Var(3), dict.parse("ℕ").unwrap()),

                // T
                (TypeID::Var(0), dict.parse("ℕ~<Seq Char>").unwrap())
            ].into_iter().collect()
        )
    );
}

