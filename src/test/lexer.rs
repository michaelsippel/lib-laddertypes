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

use crate::{lexer::*};
use tiny_diagnostics::InputRegionTag;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_lexer_symbol() {
    let mut lex = LadderTypeLexer::from("symbol".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin:0, end:6 },
        Ok(LadderTypeToken::Symbol("symbol".into()))
    )) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num() {
    let mut lex = LadderTypeLexer::from("1234".chars());

    assert_eq!( lex.next(), Some(
        (
            InputRegionTag{ begin:0, end:4 },
            Ok(LadderTypeToken::Num(1234))
        )
    ) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_num_error() {
    let mut lex = LadderTypeLexer::from("123xxx".chars());
    assert_eq!( lex.next(), Some(
        (InputRegionTag{ begin:0, end: 4}, Err(LexError::InvalidDigit))
    ) );
}

#[test]
fn test_lexer_char() {
    let mut lex = LadderTypeLexer::from("'x'".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{begin:0, end:3}, Ok(LadderTypeToken::Char('x')))) );
    assert_eq!( lex.next(), None );

    assert_eq!( LadderTypeLexer::from("'\\0'".chars()).next(), Some((InputRegionTag{begin:0, end:4}, Ok(LadderTypeToken::Char('\0')))) );
    assert_eq!( LadderTypeLexer::from("'\\n'".chars()).next(), Some((InputRegionTag{begin:0, end:4}, Ok(LadderTypeToken::Char('\n')))) );
    assert_eq!( LadderTypeLexer::from("'\\t'".chars()).next(), Some((InputRegionTag{begin:0, end:4}, Ok(LadderTypeToken::Char('\t')))) );
    assert_eq!( LadderTypeLexer::from("'\\\''".chars()).next(), Some((InputRegionTag{begin:0, end:4}, Ok(LadderTypeToken::Char('\'')))) );
    assert_eq!( LadderTypeLexer::from("'\\\\'".chars()).next(), Some((InputRegionTag{begin:0, end:4}, Ok(LadderTypeToken::Char('\\')))) );
}

#[test]
fn test_lexer_char_error() {
    let mut lex = LadderTypeLexer::from("'xx'".chars());
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end:3 }, Err(LexError::InvalidChar))) );
}

#[test]
fn test_lexer_ladder() {
    let mut lex = LadderTypeLexer::from("abc~def".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 3 }, Ok(LadderTypeToken::Symbol("abc".into())))) );
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 3, end: 4 }, Ok(LadderTypeToken::Ladder))) );
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 4, end: 7 }, Ok(LadderTypeToken::Symbol("def".into())))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_ladder_space() {
    let mut lex = LadderTypeLexer::from("abc   ~ def".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 3 }, Ok(LadderTypeToken::Symbol("abc".into())))) );
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 6, end: 7 }, Ok(LadderTypeToken::Ladder))) );
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 8, end: 11 }, Ok(LadderTypeToken::Symbol("def".into())))) );
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app() {
    let mut lex = LadderTypeLexer::from("<Seq Char>".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 1  }, Ok(LadderTypeToken::OpenSpec)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 1, end: 4  }, Ok(LadderTypeToken::Symbol("Seq".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 5, end: 9  }, Ok(LadderTypeToken::Symbol("Char".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 9, end: 10  }, Ok(LadderTypeToken::CloseSpec)) ));
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_app_space() {
    let mut lex = LadderTypeLexer::from("   <Seq      Char  >".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 3, end: 4  }, Ok(LadderTypeToken::OpenSpec)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 4, end: 7  }, Ok(LadderTypeToken::Symbol("Seq".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 13, end: 17  }, Ok(LadderTypeToken::Symbol("Char".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 19, end: 20  }, Ok(LadderTypeToken::CloseSpec)) ));
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_constraints() {
    let mut lex = LadderTypeLexer::from(":<= :>< :||".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 3  }, Ok(LadderTypeToken::SubType)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 4, end: 7  }, Ok(LadderTypeToken::TraitType)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 8, end: 11  }, Ok(LadderTypeToken::ParallelType)) ));
}

#[test]
fn test_lexer_arrows() {
    let mut lex = LadderTypeLexer::from(" -->  -morph-> ".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 1, end: 4  }, Ok(LadderTypeToken::ArrowFunc)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 6, end: 14  }, Ok(LadderTypeToken::ArrowMorph)) ));
}

#[test]
fn test_lexer_struct() {
    let mut lex = LadderTypeLexer::from("{ a: { |x:X |y:Y }; b: B; }".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 1  }, Ok(LadderTypeToken::OpenStruct)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 2, end: 3  }, Ok(LadderTypeToken::Symbol("a".into())))));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 3, end: 4  }, Ok(LadderTypeToken::AssignType)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 5, end: 6  }, Ok(LadderTypeToken::OpenStruct)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 7, end: 8  }, Ok(LadderTypeToken::EnumSep)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 8, end: 9  }, Ok(LadderTypeToken::Symbol("x".into())))));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 9, end: 10  }, Ok(LadderTypeToken::AssignType)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 10, end: 11  }, Ok(LadderTypeToken::Symbol("X".into())))));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 12, end: 13  }, Ok(LadderTypeToken::EnumSep)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 13, end: 14  }, Ok(LadderTypeToken::Symbol("y".into())))));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 14, end: 15  }, Ok(LadderTypeToken::AssignType)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 15, end: 16  }, Ok(LadderTypeToken::Symbol("Y".into())))));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 17, end: 18  }, Ok(LadderTypeToken::CloseStruct)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 18, end: 19  }, Ok(LadderTypeToken::StructSep)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 20, end: 21  }, Ok(LadderTypeToken::Symbol("b".into())))));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 21, end: 22  }, Ok(LadderTypeToken::AssignType)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 23, end: 24  }, Ok(LadderTypeToken::Symbol("B".into())))));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 24, end: 25  }, Ok(LadderTypeToken::StructSep)) ));

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 26, end: 27  }, Ok(LadderTypeToken::CloseStruct)) ));
    assert_eq!( lex.next(), None );
}

#[test]
fn test_lexer_univ() {
    let mut lex = LadderTypeLexer::from("∀(α:<=A)".chars());

    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 0, end: 1  }, Ok(LadderTypeToken::Univ)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 1, end: 2  }, Ok(LadderTypeToken::Open)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 2, end: 3  }, Ok(LadderTypeToken::Symbol("α".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 3, end: 6  }, Ok(LadderTypeToken::SubType)) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 6, end: 7  }, Ok(LadderTypeToken::Symbol("A".into()))) ));
    assert_eq!( lex.next(), Some((InputRegionTag{ begin: 7, end: 8  }, Ok(LadderTypeToken::Close)) ));
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

    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Date".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("TimeSince".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("UnixEpoch".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Duration".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Seconds".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("ℕ".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("PosInt".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Num(10))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("BigEndian".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Digit".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Num(10))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("SepSeq".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Char(':'))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Unicode".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("UTF-8".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Ladder)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::OpenSpec)));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Seq".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::Symbol("Byte".into()))));
    assert_eq!( lex.next().map(|r| r.1), Some(Ok(LadderTypeToken::CloseSpec)));

    assert_eq!( lex.next(), None );
}
