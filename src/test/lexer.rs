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

use crate::lexer::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_lexer_symbol() {
    let mut lex = LadderTypeLexer::from("symbol".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("symbol".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num() {
    let mut lex = LadderTypeLexer::from("1234".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(1234))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num_error() {
    let mut lex = LadderTypeLexer::from("123xxx".chars());
    assert_eq!( lex.next(), Some(Err(LexError::InvalidDigit)) );
}

#[test]
fn test_lexer_char() {
    let mut lex = LadderTypeLexer::from("'x'".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Char('x'))) );
    assert_eq!( lex.next(), None );

    assert_eq!( LadderTypeLexer::from("'\\0'".chars()).next(), Some(Ok(LadderTypeToken::Char('\0'))) );
    assert_eq!( LadderTypeLexer::from("'\\n'".chars()).next(), Some(Ok(LadderTypeToken::Char('\n'))) );
    assert_eq!( LadderTypeLexer::from("'\\t'".chars()).next(), Some(Ok(LadderTypeToken::Char('\t'))) );
    assert_eq!( LadderTypeLexer::from("'\\\''".chars()).next(), Some(Ok(LadderTypeToken::Char('\''))) );
    assert_eq!( LadderTypeLexer::from("'\\\\'".chars()).next(), Some(Ok(LadderTypeToken::Char('\\'))) );
}

#[test]
fn test_lexer_char_error() {
    let mut lex = LadderTypeLexer::from("'xx'".chars());
    assert_eq!( lex.next(), Some(Err(LexError::InvalidChar)) );
}

#[test]
fn test_lexer_ladder() {
    let mut lex = LadderTypeLexer::from("abc~def".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_ladder_space() {
    let mut lex = LadderTypeLexer::from("abc   ~ def".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("abc".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("def".into()))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app() {
    let mut lex = LadderTypeLexer::from("<Seq Char>".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app_space() {
    let mut lex = LadderTypeLexer::from("   <Seq      Char  >".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Char".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_constraints() {
    let mut lex = LadderTypeLexer::from(":<= :>< :||".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::SubType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::TraitType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::ParallelType)) );
}

#[test]
fn test_lexer_arrows() {
    let mut lex = LadderTypeLexer::from(" -->  -morph-> ".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::ArrowFunc)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::ArrowMorph)) );
}

#[test]
fn test_lexer_struct() {
    let mut lex = LadderTypeLexer::from("{ a: { |x:X |y:Y }; b: B; }".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenStruct)) );

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("a".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::AssignType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenStruct)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::EnumSep)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("x".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::AssignType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("X".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::EnumSep)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("y".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::AssignType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Y".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseStruct)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::StructSep)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("b".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::AssignType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("B".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::StructSep)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseStruct)) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_univ() {
    let mut lex = LadderTypeLexer::from("∀(α:<=A)".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Univ)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Open)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("α".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::SubType)) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("A".into()))) );
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Close)) );
    assert_eq!( lex.next(), None );
}


#[test]
fn test_lexer_large() {
    let mut lex = LadderTypeLexer::from(
        "<Seq Date
              ~<TimeSince UnixEpoch>
              ~<Duration Seconds>
              ~ℕ
              ~<PosInt 10 BigEndian>
              ~< Seq <Digit 10>~Unicode > >
         ~<SepSeq Unicode ':'>
         ~<Seq Unicode>
         ~UTF-8
         ~<Seq Byte>".chars());

    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Date".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("TimeSince".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("UnixEpoch".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Duration".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seconds".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("ℕ".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("PosInt".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(10))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("BigEndian".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Digit".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Num(10))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("SepSeq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Char(':'))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("UTF-8".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::Symbol("Byte".into()))));
    assert_eq!( lex.next(), Some(Ok(LadderTypeToken::CloseSpec)));

    assert_eq!( lex.next(), None );
}
