
use {
    crate::{dict::*, term::*},
    std::iter::FromIterator
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_subst() {
    let mut dict = TypeDict::new();

    let mut σ = std::collections::HashMap::new();

    // T  -->  ℕ
    σ.insert
        (dict.add_varname(String::from("T")),
         dict.parse("ℕ").unwrap());

    // U  -->  <Seq Char>
    σ.insert
        (dict.add_varname(String::from("U")),
         dict.parse("<Seq Char>").unwrap());


    assert_eq!(
        dict.parse("<Seq T~U>").unwrap()
            .apply_substitution(&|typid|{ σ.get(typid).cloned() }).clone(),
        dict.parse("<Seq ℕ~<Seq Char>>").unwrap()
    );
}

