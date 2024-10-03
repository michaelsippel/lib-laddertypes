
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

#[test]
fn test_subtype_unification() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname(String::from("T"));
    dict.add_varname(String::from("U"));
    dict.add_varname(String::from("V"));
    dict.add_varname(String::from("W"));

    assert_eq!(
        UnificationProblem::new(vec![
            (dict.parse("<Seq~T <Digit 10> ~ Char>").unwrap(),
                dict.parse("<Seq~<LengthPrefix x86.UInt64> Char ~ Ascii>").unwrap()),
        ]).solve_subtype(),
        Ok((
            dict.parse("<Seq <Digit 10>>").unwrap(),
            vec![
                // T
                (TypeID::Var(0), dict.parse("<LengthPrefix x86.UInt64>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        UnificationProblem::new(vec![
            (dict.parse("U").unwrap(), dict.parse("<Seq Char>").unwrap()),
            (dict.parse("T").unwrap(), dict.parse("<Seq U>").unwrap()),
        ]).solve_subtype(),
        Ok((
            TypeTerm::unit(),
            vec![
                // T
                (TypeID::Var(0), dict.parse("<Seq <Seq Char>>").unwrap()),

                // U
                (TypeID::Var(1), dict.parse("<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        UnificationProblem::new(vec![
            (dict.parse("<Seq T>").unwrap(),
                dict.parse("<Seq W~<Seq Char>>").unwrap()),
            (dict.parse("<Seq~<LengthPrefix x86.UInt64> ℕ~<PosInt 10 BigEndian>>").unwrap(),
                dict.parse("<<LengthPrefix x86.UInt64> W>").unwrap()),
        ]).solve_subtype(),
        Ok((
            dict.parse("
                <Seq ℕ~<PosInt 10 BigEndian>>
            ").unwrap(),
            vec![
                // W
                (TypeID::Var(3), dict.parse("ℕ~<PosInt 10 BigEndian>").unwrap()),

                // T
                (TypeID::Var(0), dict.parse("ℕ~<PosInt 10 BigEndian>~<Seq Char>").unwrap())
            ].into_iter().collect()
        ))
    );
}
