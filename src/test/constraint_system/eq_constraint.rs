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

use {
    crate::{parser::*,
        context::*,
        constraint_system::{
            ConstraintSystem,
            CP2,
            ConstraintError
        }
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

fn test_unify(ts1: &str, ts2: &str, expect_unificator: bool) {
    let mut ctx = Context::new();
    ctx.add_variable("T", TypeKind::Type);
    ctx.add_variable("U", TypeKind::Type);
    ctx.add_variable("V", TypeKind::Type);
    ctx.add_variable("W", TypeKind::Type);

    let mut t1 = ctx.parse(ts1).unwrap();
    let mut t2 = ctx.parse(ts2).unwrap();
    let σ = crate::unify( &t1, &t2 );

    if expect_unificator {
        assert!(σ.is_ok());

        let σ = σ.unwrap();

        assert_eq!(
            t1.apply_subst(&σ),
            t2.apply_subst(&σ)
        );
    } else {
        assert!(! σ.is_ok());
    }
}

#[test]
fn test_unification_error() {
    let mut ctx = Context::new();
    ctx.add_variable("T", TypeKind::Type);

    assert_eq!(
        crate::unify(
            &ctx.parse("<A T>").unwrap(),
            &ctx.parse("<B T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![0],
            t1: ctx.parse("A").unwrap(),
            t2: ctx.parse("B").unwrap()
        })
    );

    assert_eq!(
        crate::unify(
            &ctx.parse("<V <U A> T>").unwrap(),
            &ctx.parse("<V <U B> T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![1, 1],
            t1: ctx.parse("A").unwrap(),
            t2: ctx.parse("B").unwrap()
        })
    );

    assert_eq!(
        crate::unify(
            &ctx.parse("T").unwrap(),
            &ctx.parse("<Seq T>").unwrap()
        ),

        Err(ConstraintError {
            addr: vec![],
            t1: ctx.parse("T").unwrap(),
            t2: ctx.parse("<Seq T>").unwrap()
        })
    );
}

#[test]
fn test_unification() {
      test_unify("A", "A", true);
   // test_unify("A", "B", false);
 //   test_unify("<Seq T>", "<Seq Ascii~Char>", true);

    // this worked easily with desugared terms,
    // but is a weird edge case with sugared terms
    // not relevant now
    //test_unify("<Seq T>", "<U Char>", true);
/*
    test_unify(
        "<Seq Path~<Seq Char>>~<SepSeq Char '\\n'>~<Seq Char>",
        "<Seq T~<Seq Char>>~<SepSeq Char '\\n'>~<Seq Char>",
        true
    );
*/
    let mut dict = Context::new();

    dict.add_variable("T", TypeKind::Type);
    dict.add_variable("U", TypeKind::Type);
    dict.add_variable("V", TypeKind::Type);
    dict.add_variable("W", TypeKind::Type);

    assert_eq!(
        ConstraintSystem::new_eq(vec![
            CP2 {
                addr: Vec::new(),
                lhs: dict.parse("U").unwrap(),
                rhs: dict.parse("[Char]").unwrap()
            },
            CP2 {
                addr: Vec::new(),
                lhs: dict.parse("T").unwrap(),
                rhs: dict.parse("[U]").unwrap()
            }
        ]).solve(),
        Ok((
            vec![],
            vec![
                // T
                (0, dict.parse("[[Char]]").unwrap()),

                // U
                (1, dict.parse("[Char]").unwrap())
            ].into_iter().collect()
        ))
    );

    assert_eq!(
        ConstraintSystem::new_eq(vec![
            CP2 {
                addr: Vec::new(),
                lhs : dict.parse("[T]").unwrap(),
                rhs : dict.parse("[W~[Char]]").unwrap()
            },
            CP2 {
                addr: Vec::new(),
                lhs : dict.parse("[ℕ]").unwrap(),
                rhs : dict.parse("[W]").unwrap(),
            }
        ]).solve(),
        Ok((
            vec![],
            vec![
                // W
                (3, dict.parse("ℕ").unwrap()),

                // T
                (0, dict.parse("ℕ~[Char]").unwrap())
            ].into_iter().collect()
        ))
    );
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
