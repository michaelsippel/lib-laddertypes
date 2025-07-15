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
    crate::{parser::*, Context}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_curry() {
    let mut dict = Context::new();

    assert_eq!(
        dict.parse("<A B C>").unwrap().curry(),
        dict.parse("<<A B> C>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C D>").unwrap().curry(),
        dict.parse("<<<A B> C> D>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C D E F G H I J K>").unwrap().curry(),
        dict.parse("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap()
    );

    assert_eq!(
        dict.parse("<A~X B C>").unwrap().curry(),
        dict.parse("<<A~X B> C>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C~Y~Z> ~ K").unwrap().curry(),
        dict.parse("< <A B> C~Y~Z > ~ K").unwrap()
    );
}

#[test]
fn test_decurry() {
    let mut dict = Context::new();

    assert_eq!(
        dict.parse("<<A B> C>").unwrap().decurry(),
        dict.parse("<A B C>").unwrap()
    );
    assert_eq!(
        dict.parse("<<<A B> C> D>").unwrap().decurry(),
        dict.parse("<A B C D>").unwrap(),
    );
    assert_eq!(
        dict.parse("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap().decurry(),
        dict.parse("<A B C D E F G H I J K>").unwrap()
    );

    assert_eq!(
        dict.parse("<<A~X B> C>").unwrap().decurry(),
        dict.parse("<A~X B C>").unwrap()
    );
    assert_eq!(
        dict.parse("<<A~X B> C~Y>~K").unwrap().decurry(),
        dict.parse("<A~X B C~Y> ~K").unwrap()
    );
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
