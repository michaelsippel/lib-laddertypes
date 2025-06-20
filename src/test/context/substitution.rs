
use {
    crate::{dict::*, parser::*, Context, LayeredContext, TypeKind,}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_subst() {
    let mut dict = Context::new();

    let mut σ = std::collections::HashMap::new();

    // T  -->  ℕ
    σ.insert(dict.add_variable("T", TypeKind::Type), dict.parse("ℕ").unwrap());

    // U  -->  <Seq Char>
    σ.insert(dict.add_variable("U", TypeKind::Type), dict.parse("<Seq Char>").unwrap());

    assert_eq!(
        dict.parse("<Seq T~U>").unwrap().apply_subst(&σ).clone(),
        dict.parse("<Seq ℕ~<Seq Char>>").unwrap()
    );
}
