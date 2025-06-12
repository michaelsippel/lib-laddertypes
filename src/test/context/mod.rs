pub mod substitution;

use crate::{
    context::{Context, LayeredContext, TypeKind},
    parser::*,
    term::TypeTerm, TypeDict, TypeID
};

#[test]
fn test_context() {


    /*
     * Set up variable scopes
     */

    let root_ctx = Context::new();
    root_ctx.add_variable(
        "DstRadix",
        TypeKind::ValueUInt
    );

    let mut sub1_ctx = root_ctx.scope();
    assert_eq!( sub1_ctx.add_variable("Radix", TypeKind::ValueUInt), 0 );

    let mut sub2_ctx = root_ctx.scope();
    assert_eq!( sub2_ctx.add_variable("SrcRadix", TypeKind::ValueUInt), 0 );



    /*
     * check variable IDs
     */

    assert_eq!( sub1_ctx.get_typeid("Radix"), Some(TypeID::Var(0)) );
    assert_eq!( sub1_ctx.get_typeid("DstRadix"), Some(TypeID::Var(1)) );

    assert_eq!( sub2_ctx.get_typeid("SrcRadix"), Some(TypeID::Var(0)) );
    assert_eq!( sub2_ctx.get_typeid("DstRadix"), Some(TypeID::Var(1)) );

    assert_eq!( sub1_ctx.parse("Radix"), Ok(TypeTerm::Var(0)) );



    /*
     * assign variables in scoped context
     */

    // Radix
    sub1_ctx.bind(0, TypeTerm::Num(10));

    // SrcRadix
    sub2_ctx.bind(0, TypeTerm::Num(10));

    // Dst Radix
    sub2_ctx.bind(1, TypeTerm::Num(16));



    /*
     * test that bound variables are substituted
     */

    assert_eq!(
        sub1_ctx
            .parse("<PosInt Radix LittleEndian> ~ <Seq <Digit Radix>>").expect("parse error")
            .apply_subst(&sub1_ctx).clone(),

        sub1_ctx
            .parse("<PosInt 10 LittleEndian> ~ <Seq <Digit 10>>").expect("parse error")
    );

    assert_eq!(
        sub2_ctx
            .parse("<PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>>").expect("parse error")
            .apply_subst(&sub1_ctx).clone(),

        sub2_ctx
            .parse("<PosInt 10 LittleEndian> ~ <Seq <Digit 10>>").expect("parse error")
    );

    assert_eq!(
        sub2_ctx
            .parse("<PosInt DstRadix LittleEndian>").expect("parse error")
            .apply_subst(&sub1_ctx).clone(),

        sub2_ctx
            .parse("<PosInt 16 LittleEndian>").expect("parse error")
    );
}
