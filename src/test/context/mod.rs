/*
   lib-laddertypes
   Copyright (C) 2023-2025  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

pub mod substitution;

use crate::{
    context::{Context, LayeredContext, TypeKind}, parser::*, term::TypeTerm, ConstraintSystem, ContextEntry, MorphismType, TypeDict, TypeID, CP2
};

#[test]
fn test_context() {


    /*
     * Set up variable scopes
     */

    let root_ctx = Context::new();
    root_ctx.add_variable(
        "DstRadix",
        TypeKind::Value(root_ctx.clone().parse("ℕ").expect("parse"))
    );

    let mut sub1_ctx = root_ctx.scope();
    assert_eq!( sub1_ctx.add_variable("Radix", TypeKind::Value(sub1_ctx.clone().parse("ℕ").expect("parse"))), 0 );

    let mut sub2_ctx = root_ctx.scope();
    assert_eq!( sub2_ctx.add_variable("SrcRadix", TypeKind::Value(sub2_ctx.clone().parse("ℕ").expect("parse"))), 0 );



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
            .apply_subst(&sub2_ctx).clone(),

        sub2_ctx
            .parse("<PosInt 16 LittleEndian>").expect("parse error")
    );
}




#[test]
fn test_morphism_compat() {
    let mut ctx = Context::new();

    let mut c1 = ctx.scope();
    c1.add_variable("T1", TypeKind::Type);
    c1.add_variable("T2", TypeKind::Type);
    let t1 = MorphismType {
        Γ: vec![
            ContextEntry{ symbol: "T1".into(), kind: TypeKind::Type},
            ContextEntry{ symbol: "T2".into(), kind: TypeKind::Type},
        ],
        bounds: Vec::new(),
        src_type: c1.parse("<Seq T1>~<A T1 T2>").unwrap(),
        dst_type: c1.parse("<Seq T1>~<B T2 T2>").unwrap()
    };

    let mut c2 = ctx.scope();
    c2.add_variable("S1", TypeKind::Type);
    c2.add_variable("T1", TypeKind::Type); //< this variable name is scoped thus a *different* variable than T1 from t1
    let t2 = MorphismType {
        Γ: vec![
            ContextEntry{ symbol: "S1".into(), kind: TypeKind::Type},
            ContextEntry{ symbol: "T1".into(), kind: TypeKind::Type},
        ],
        bounds: Vec::new(),
        src_type: c2.parse("<Seq NotT>~<B S1 T1>").unwrap(),
        dst_type: c2.parse("<Seq NotT>~<C T1>").unwrap()
    };

    // pull t1 & t2 into root ctx
    let mut t1 = t1.dst_type.clone();
    t1.apply_subst(&ctx.shift_variables(&c1));
    let mut t2 = t2.src_type.clone();
    t2.apply_subst(&ctx.shift_variables(&c2));

    let csp = ConstraintSystem::new_sub(vec![
        CP2 {
            lhs: t1.clone(),
            rhs: t2.clone(),
            addr: vec![]
        }
    ]);

    eprintln!("t1 = {:?} = {}", t1, t1.pretty(&mut ctx.clone(), 0));
    eprintln!("t2 = {:?} = {}", t2, t2.pretty(&mut ctx.clone(), 0));

    match csp.solve() {
        Ok((Ψ,σ)) => {
            eprintln!("σ = {:?}", σ);
            assert!(true);
        }
        Err(err) => {
            assert!(false);
        }
    }
}
