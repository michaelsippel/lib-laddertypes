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
fn test_trait_bound1() {
    let mut dict = BimapTypeDict::new();

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
    let mut dict = BimapTypeDict::new();

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
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        ConstraintSystem::new_trait(vec![
            ConstraintPair {
                addr: Vec::new(),
                lhs : dict.parse("<Struct <a S~A> <b T~B>>").unwrap(),
                rhs : dict.parse("<Struct <a S> <b T>>").unwrap()
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
                lhs : dict.parse("<Struct <a S~A> <b T~B>>").unwrap(),
                rhs : dict.parse("<Struct <a S>>").unwrap()
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
                lhs : dict.parse("<Struct <a S~A> <b T~B>>").unwrap(),
                rhs : dict.parse("<Struct <a A>>").unwrap()
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
                lhs : dict.parse("<Struct <a S~A> <b T~B>>").unwrap(),
                rhs : dict.parse("<Struct <a T>>").unwrap()
            }
        ]).solve(),
        Err(ConstraintError { addr: vec![0], t1: dict.parse("S~A").unwrap(), t2: dict.parse("T").unwrap() })
    );
}
