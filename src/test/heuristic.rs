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

use crate::{dict::*, heuristic::*, morphism::*, parser::*, Context};

#[test]
fn test_heuristic() {
    let mut dict = Context::new();

    assert_eq!(
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: dict.parse("A").expect("parse"),
            dst_type: dict.parse("A").expect("parse")
        }.estimated_cost(),
        0
    );

    assert_eq!(
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: dict.parse("<Digit 10> ~ Char ~ Ascii ~ native.UInt8").expect("parse"),
            dst_type: dict.parse("<Digit 16> ~ native.UInt8").expect("parse")
        }.estimated_cost(),
        40
    );
}
