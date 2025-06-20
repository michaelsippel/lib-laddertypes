use {
    crate::{constraint_system::{
            ConstraintError, ConstraintPair, ConstraintSystem
        }, dict::*, parser::*, Context
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_trait_bound1() {
    let mut dict = Context::new();

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B").unwrap(),
                rhs : dict.parse("A").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ B").unwrap(),
                rhs : dict.parse("B").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A").unwrap(),
                rhs : dict.parse("B").unwrap()
            }
        ]).solve(),
        Err(ConstraintError { addr: vec![], t1: dict.parse("A").unwrap(), t2: dict.parse("B").unwrap() })
    );
    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A").unwrap(),
                rhs : dict.parse("A~B").unwrap()
            }
        ]).solve(),
        Err(ConstraintError { addr: vec![], t1: dict.parse("A").unwrap(), t2: dict.parse("A~B").unwrap() })
    );
}

#[test]
fn test_trait_bound_spec() {
    let mut dict = Context::new();

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("A ~ <B~C D> ~ E").unwrap(),
                rhs : dict.parse("<B D>").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );
}

#[test]
fn test_trait_bound_struct() {
    let mut dict = Context::new();

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("{ a:S~A; b:T~B; }").unwrap(),
                rhs : dict.parse("{ a:S; b:T; }").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("{ a: S; b: T~B; }").unwrap(),
                rhs : dict.parse("{ a: S; }").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("{ a: S~A; b: T~B; }").unwrap(),
                rhs : dict.parse("{ a: A; }").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("{ a: S~A; b: T~B; }").unwrap(),
                rhs : dict.parse("{ a: T; }").unwrap()
            }
        ]).solve(),
        Err(ConstraintError { addr: vec![0], t1: dict.parse("S~A").unwrap(), t2: dict.parse("T").unwrap() })
    );
}
