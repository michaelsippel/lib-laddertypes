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

#[test]
fn test_mixed_trait_bounds() {
    let mut dict = BimapTypeDict::new();

    dict.add_varname("T".into());
    dict.add_varname("Radix".into());

    assert_eq!(
        ConstraintSystem::new(
            // eq constraints
            vec![],

            // sub constraints
            vec![
                ConstraintPair {
                    addr: Vec::new(),
                    lhs : dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10>~Char~native.UInt8>").unwrap(),
                    rhs : dict.parse("<Seq T>").unwrap()
                }
            ],

            // trait constraints
            vec![
                ConstraintPair {
                    addr: Vec::new(),
                    lhs: dict.parse("T").unwrap(),
                    rhs: dict.parse("<Digit Radix>").unwrap(),
                }
            ],

            // parallel constraints
            vec![]
        ).solve(),
        Ok((
            // ψ
            vec![
                dict.parse("ℕ ~ <PosInt 10 BigEndian>").unwrap()
            ],

            // σ
            vec![
                (dict.get_typeid(&"T".into()).unwrap(), dict.parse("<Digit 10>~Char~native.UInt8").unwrap()),
                (dict.get_typeid(&"Radix".into()).unwrap(), dict.parse("10").unwrap())
            ].into_iter().collect()
        ))
    );
}
