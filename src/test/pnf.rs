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

use crate::{Context, parser::*};

#[test]
fn test_normalize_id() {
    let mut dict = Context::new();

    assert_eq!(
        dict.parse("A~B~C").expect("parse error"),
        dict.parse("A~B~C").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A B>~C").expect("parse error"),
        dict.parse("<A B>~C").expect("parse error").normalize(),
    );
}

#[test]
fn test_normalize_spec() {
    let mut dict = Context::new();

    assert_eq!(
        dict.parse("<A B~C>").expect("parse error"),
        dict.parse("<A B>~<A C>").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A~Y B>").expect("parse error"),
        dict.parse("<A~Y B>~<Y B>").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A B~C D~E>").expect("parse error"),
        dict.parse("<A B D>~<A C D>~<A C E>").expect("parse errror").normalize(),
    );

    assert_eq!(
        dict.parse("<A~X B~C D~E>").expect("parse error"),
        dict.parse("<A~X B D>~<A~X B~C E>~<X C E>").expect("parse errror").normalize(),
    );
}

#[test]
fn test_normalize_seq() {
    let mut dict = Context::new();
    assert_eq!(
        dict.parse("[Char~Ascii]").expect("parse error"),
        dict.parse("[Char]~[Ascii]").expect("parse errror").normalize(),
    );

    eprintln!("---------------");
    assert_eq!(
        dict.parse("[<Digit 10>~Char]").expect("parse error"),
        dict.parse("[<Digit 10>]~[Char]").expect("parse errror").normalize(),
    );
    eprintln!("---------------");
    assert_eq!(
        dict.parse("[~<ValueDelim '\\0'> Char~Ascii~native.UInt8]").expect("parse error"),
        dict.parse("[Char] ~ <<ValueDelim '\\0'> Char> ~ <<ValueDelim '\\0'> Ascii~native.UInt8>").expect("parse error").normalize(),
    );
    eprintln!("---------------");
    assert_eq!(
        dict.parse("[~<ValueDelim '\\0'> Char~Ascii~native.UInt8]").expect("parse error"),
        dict.parse("[Char~Ascii] ~ <<ValueDelim '\\0'> Char~Ascii> ~ <<ValueDelim '\\0'> native.UInt8>").expect("parse error").normalize(),
    );
}

#[test]
fn test_normalize_complex_spec() {
    let mut dict = Context::new();
    assert_eq!(
        dict.parse("<A~Y <B C~D~E> F H H>").expect("parse error"),
        dict.parse("<A~Y <B C> F H H>
                   ~<A~Y <B D> F H H>
                   ~<Y <B E> F H H>").expect("parse errror")
               .normalize(),
    );
}

#[test]
fn test_normalize_struct() {
    let mut dict = Context::new();
    assert_eq!(
        dict.parse("{~Aligned
                a: TimePoint~<TimeSince UnixEpoch>~Seconds~native.UInt64;
                b: Angle ~ Degrees ~ ℝ ~ native.Float32;
            }
            ").expect("parse error"),
        dict.parse("
            { a: TimePoint; b: Angle; }
        ~   { a: <TimeSince UnixEpoch>~Seconds; b: Angle~Degrees~ℝ; }
        ~   {~Aligned a: native.UInt64; b: native.Float32; }
        ").expect("parse error")

            .normalize(),
    );
}

#[test]
fn test_normalize_enum() {
    let mut dict = Context::new();
    assert_eq!(
        dict.parse("< Enum
                <  a   TimePoint~<TimeSince UnixEpoch>~Seconds~native.UInt64  >
                <  b   Angle ~ Degrees ~ ℝ ~ native.Float32 >
            >
            ").expect("parse error"),
        dict.parse("
            < Enum <a TimePoint> <b Angle> >
        ~   < Enum  <a <TimeSince UnixEpoch>~Seconds> <b Angle~Degrees~ℝ> >
        ~   < Enum  <a native.UInt64> <b native.Float32> >
        ").expect("parse errror")

            .normalize(),
    );
}
