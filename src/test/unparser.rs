/*
   lib-laddertypes
   Copyright (C) 2026  Michael Sippel
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
    crate::{parser::ParseLadderType, unparser::UnparseLadderType, Context}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_unparser_ladder() {
    let mut ctx = Context::new();
    let t = ctx.parse("A ~ B ~ C ~ def").unwrap();
    let t_unparsed = ctx.unparse(&t);
    assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
}

#[test]
fn test_unparser_app() {
    let mut ctx = Context::new();
    let t = ctx.parse("<A <B C> D E~F>").unwrap();
    let t_unparsed = ctx.unparse(&t);
    assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
}

#[test]
fn test_unparser_struct() {
    let mut ctx = Context::new();
    {
        let t = ctx.parse("{ x: X; y: Y; }").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
    {
        let t = ctx.parse("{~S x: X; y: Y; }").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
}

#[test]
fn test_unparser_enum() {
    let mut ctx = Context::new();
    {
        let t = ctx.parse("{ | x: X | y: Y }").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
    {
        let t = ctx.parse("{~S | x: X | y: Y }").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
}

#[test]
fn test_unparser_seq() {
    let mut ctx = Context::new();
    {
        let t = ctx.parse("[ T ]").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
    {
        let t = ctx.parse("[~S T ]").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
}

#[test]
fn test_unparser_arrow() {
    let mut ctx = Context::new();
    {
        let t = ctx.parse("A --> B").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
    {
        let t = ctx.parse("<Some Very~Fancy Type <With 123>> --> And ~ [Arrows]").unwrap();
        let t_unparsed = ctx.unparse(&t);
        assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
    }
}

/*
#[test]
fn test_unparser_univ() {
    let mut ctx = Context::new();
    let t = ctx.parse("∀T  [~<LengthPrefix native.UInt32> T] --> [~<LengthPrefix native.UInt64> T]").unwrap();
    let t_unparsed = ctx.unparse(&t);
    eprintln!("{t_unparsed}");
    //assert_eq!( ctx.parse(&t_unparsed).unwrap(), t );
}

*/