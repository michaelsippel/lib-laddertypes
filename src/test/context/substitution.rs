
use {
    crate::{dict::*, parser::*,}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_subst() {
    let mut dict = BimapTypeDict::new();

    let mut σ = std::collections::HashMap::new();

    // T  -->  ℕ
    σ.insert
        (dict.add_varname("T"),
         dict.parse_desugared("ℕ").unwrap().sugar(&mut dict));

    // U  -->  <Seq Char>
    σ.insert
        (dict.add_varname("U"),
         dict.parse_desugared("<Seq Char>").unwrap().sugar(&mut dict));


    assert_eq!(
        dict.parse_desugared("<Seq T~U>").unwrap().sugar(&mut dict).apply_subst(&σ).clone(),
        dict.parse_desugared("<Seq ℕ~<Seq Char>>").unwrap().sugar(&mut dict)
    );
}
