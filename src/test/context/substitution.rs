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
