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
    crate::{
        context::dict::*, lexer::*, term::*, LayeredContext, TypeKind
    },
    std::iter::Peekable,
    tiny_diagnostics::InputRegionTag,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedClose,
    UnexpectedLadder,
    UnexpectedEnd,
    UnexpectedToken,
}

pub enum ParseInfoType {
    UnknownTypeName( String )
}

pub struct ParseInfo {
    pub char_range: InputRegionTag,
    pub info: ParseInfoType
}

type ParseLadderTypeResult = Result<(InputRegionTag, TypeTerm), (InputRegionTag, ParseError)>;

pub trait ParseLadderType {
    fn parse(&mut self, s: &str) -> Result<TypeTerm, (InputRegionTag, ParseError)>;
    fn parse_warn(&mut self, s:&str, warnings: &mut Vec<ParseInfo>) -> Result<TypeTerm, (InputRegionTag, ParseError)>;

    fn parse_top<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_app<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_rung<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_ladder<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_seq<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_struct<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;

    fn parse_univ<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<T: LayeredContext> ParseLadderType for T {
    fn parse(&mut self, s: &str) -> Result<TypeTerm, (InputRegionTag, ParseError)>
    {
        let mut warnings = Vec::new();
        self.parse_warn(s, &mut warnings)
    }

    fn parse_warn(&mut self, s: &str, warnings: &mut Vec<ParseInfo>) -> Result<TypeTerm, (InputRegionTag, ParseError)> {
        let mut tokens = LadderTypeLexer::from(s.chars()).peekable();

        match self.parse_top(&mut tokens, warnings) {
            Ok((r,t)) => {
                if let Some((r_tok,_tok)) = tokens.peek() {
                    Err((*r_tok, ParseError::UnexpectedToken))
                } else {
                    Ok(t)
                }
            }
            Err((r,err)) => Err((r,err))
        }
    }

    fn parse_top<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        // 1. Ladders
        let (r_t1, t1) = self.parse_ladder(tokens, warnings)?;

        // 2. Arrows
        match tokens.peek() {
            Some((_range, Ok(LadderTypeToken::ArrowFunc))) => {
                tokens.next();
                let (r_t2, t2) = self.parse_top(tokens, warnings)?;
                return Ok((
                    InputRegionTag::max(r_t1, r_t2),
                    TypeTerm::Func(vec![
                        t1, t2
                    ])));
            }
            Some((_range, Ok(LadderTypeToken::ArrowMorph))) => {
                tokens.next();
                let (r_t2, t2) = self.parse_top(tokens, warnings)?;
                return Ok((InputRegionTag::max(r_t1, r_t2), TypeTerm::Morph(Box::new(t1), Box::new(t2))));
            }
            Some((range, Err(err))) => {
                return Err((*range, ParseError::LexError(err.clone())));
            }
            _ => {
                return Ok((r_t1, t1));
            }
        }
    }

    fn parse_app<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        let mut args = Vec::new();
        let mut r = InputRegionTag::default();

        while let Some((r_tok, tok)) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::CloseSpec) => {
                    let tok = tokens.next();
                    return Ok((r, TypeTerm::Spec(args)));
                }
                _ => {
                    match self.parse_top(tokens, warnings) {
                        Ok((r_arg,a)) => {
                            args.push(a);
                        }
                        Err((r,err)) => { return Err((r,err)); }
                    }
                }
            }
        }
        Err((r, ParseError::UnexpectedEnd))
    }

    fn parse_seq<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        let mut seq_repr = None;
        let mut r = InputRegionTag::default();

        if let Some((range, Ok(LadderTypeToken::Ladder))) = tokens.peek() {
            r = *range;
            tokens.next();
            seq_repr = Some(Box::new(self.parse_top(tokens, warnings)?.1));
        }


        while let Some((range, tok)) = tokens.peek() {
            match *tok {
                (Ok(LadderTypeToken::CloseSeq)) => {
                    return Err((*range, ParseError::UnexpectedClose));
                }
                _ => {
                    let (r_item, item) = self.parse_top(tokens, warnings)?;
                    r = InputRegionTag::max(r, r_item);
                    while let Some((next_range, tok)) = tokens.next() {
                        match tok {
                            Ok(LadderTypeToken::CloseSeq) => { return
                                Ok((
                                    InputRegionTag::max(r, next_range),
                                    TypeTerm::Seq { seq_repr, item: Box::new(item.clone()) }
                                ));
                            }
                            Ok(_) => { return Err((next_range, ParseError::UnexpectedToken)); }
                            Err(err) => { return Err((next_range, ParseError::LexError(err))); }
                        }
                    }
                }
            }
        }

        Err((r, ParseError::UnexpectedEnd))
    }

    fn parse_struct<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        let mut struct_repr = None;
        let mut is_enum = false;
        let mut variants = Vec::new();
        let mut members = Vec::new();

        let mut r = InputRegionTag::default();

        if let Some((range, Ok(LadderTypeToken::Ladder))) = tokens.peek() {
            r = *range;
            tokens.next();
            struct_repr = Some(Box::new( self.parse_top(tokens, warnings)?.1 ));
        }

        while let Some((r_tok, tok)) = tokens.peek() {
            r = InputRegionTag::max(r, *r_tok);
            match tok {
                Ok(LadderTypeToken::CloseStruct) => {
                    tokens.next();
                    if is_enum {
                        return Ok((r, TypeTerm::Enum { enum_repr: struct_repr, variants }))
                    } else {
                        return Ok((r, TypeTerm::Struct { struct_repr, members }));
                    }
                }

                Ok(LadderTypeToken::EnumSep) => {
                    tokens.next();
                    is_enum = true;

                    match tokens.next() {
                        Some((rtok, Ok(LadderTypeToken::Symbol(symbol)))) => {
                            let symbol = symbol.clone();
                            r = InputRegionTag::max(r, rtok);

                            // `:` between identifier and type
                            match tokens.next() {
                                Some((rtok2, Ok(LadderTypeToken::AssignType))) => {}
                                Some((rtok2, Err(err))) => { return Err((rtok2, ParseError::LexError(err))); }
                                Some((rtok2, _)) => { return Err((rtok2, ParseError::UnexpectedToken)); }
                                None => { return Err((r, ParseError::UnexpectedEnd)); }
                            }

                            let (rty, ty) = self.parse_top(tokens, warnings)?;
                            variants.push(EnumVariant { symbol, ty });

                            r = InputRegionTag::max(r, rty);
                        }

                        Some((rtok, Err(err))) => { return Err((rtok, ParseError::LexError(err))); }
                        Some((rtok, _)) => { return Err((rtok, ParseError::UnexpectedToken)); }
                        None => { return Err((r, ParseError::UnexpectedEnd)); }
                    }
                }
                Ok(LadderTypeToken::Symbol(symbol)) => {
                    if is_enum {
                        return Err((*r_tok, ParseError::UnexpectedToken));
                    }

                    let symbol = symbol.clone();
                    tokens.next();

                    // `:` between identifier and type
                    match tokens.next() {
                        Some((rtok, Ok(LadderTypeToken::AssignType))) => {}
                        Some((rtok, Err(err))) => { return Err((rtok, ParseError::LexError(err))); }
                        Some((rtok, _)) => { return Err((rtok, ParseError::UnexpectedToken)); }
                        None => { return Err((r, ParseError::UnexpectedEnd)); }
                    }

                    let (rty, ty) = self.parse_top(tokens, warnings)?;
                    members.push(StructMember { symbol, ty });

                    r = InputRegionTag::max(r, rty);

                    // `;` at end
                    match tokens.next() {
                        Some((rtok, Ok(LadderTypeToken::StructSep))) => {}
                        Some((rtok, Err(err))) => { return Err((rtok, ParseError::LexError(err))); }
                        Some((rtok, _)) => { return Err((rtok, ParseError::UnexpectedToken)); }
                        None => { return Err((r, ParseError::UnexpectedEnd)); }
                    }
                }

                Ok(_) => { return Err((r, ParseError::UnexpectedToken)); }
                Err(err) => { return Err((r, ParseError::LexError(err.clone()))); }
            }
        }

        Err((r, ParseError::UnexpectedEnd))
    }

    fn parse_univ<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        let mut Γ = Vec::new();
        let mut ctx = self.scope();
        let mut bounds = Vec::new();

        let mut r = InputRegionTag::default();

        // at least one symbol name follows
        while let Some((rtok, tok)) = tokens.next() {
            r = InputRegionTag::max(r, rtok);
            match tok {
                Ok(LadderTypeToken::Symbol(symbol)) => {
                    match tokens.peek() {
                        Some((rtok2, Ok(LadderTypeToken::AssignType))) => {
                            tokens.next();
                            let (rt, t) = ctx.parse_top(tokens, warnings)?;
                            r = InputRegionTag::max(r, rt);
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Value(t.clone()) });
                            ctx.add_variable( &symbol, TypeKind::Value(t) );

                            match tokens.peek() {
                                Some((rtok3, Ok(LadderTypeToken::Univ))) => {
                                    tokens.next();
                                    continue;
                                }
                                _ => { break; }
                            }
                        }
                        Some((rtok2, Ok(LadderTypeToken::Univ))) => {
                            tokens.next();
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Type });
                            ctx.add_variable( &symbol, TypeKind::Type );
                            continue;
                        }
                        Some((rtok2, Ok(_))) => {
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Type });
                            ctx.add_variable(&symbol, TypeKind::Type );
                            break; }
                        Some((rtok2, Err(err))) => { return Err((*rtok2, ParseError::LexError(err.clone()))); }
                        None => {
                            break;
                        }
                    }
                }
                Ok(_) => {
                    return Err((rtok, ParseError::UnexpectedToken));
                }
                Err(err) => { return Err((rtok, ParseError::LexError(err.clone()))); }
            }
        }

        while let Some((rtok, tok)) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Open) => {
                    // another constraint
                    tokens.next(); // take opening
                    let (rlhs, lhs) = ctx.parse_top(tokens, warnings)?;
                    r = InputRegionTag::max(r, rlhs);

                    let rel_token = tokens.next();
                    let (rrhs, rhs) = ctx.parse_top(tokens, warnings)?;
                    r = InputRegionTag::max(r, rrhs);

                    match rel_token {
                        Some((rtok2, Ok(LadderTypeToken::SubType))) => {
                            bounds.push(ConstraintPair::Subtype(lhs, rhs));
                        }
                        Some((rtok2, Ok(LadderTypeToken::TraitType))) => {
                            bounds.push(ConstraintPair::Trait(lhs, rhs));
                        }
                        Some((rtok2, Ok(LadderTypeToken::ParallelType))) => {
                            bounds.push(ConstraintPair::Parallel(lhs, rhs));
                        }
                        _ => {
                            todo!()
                        }
                    }

                    match tokens.next() {
                        Some((rtok2, Ok(LadderTypeToken::Close))) => {
                            r = InputRegionTag::max(r, rtok2);
                            continue;
                        }
                        Some((rtok2, Ok(_))) => { return Err((rtok2, ParseError::UnexpectedToken)); }
                        Some((rtok2, Err(err))) => { return Err((rtok2, ParseError::LexError(err.clone()))); }
                        None => { return Err((r, ParseError::UnexpectedEnd)); }
                    }

                }

                Ok(_) => {
                    break;
                }
                Err(err) => {
                    return Err((*rtok, ParseError::LexError(err.clone())));
                }
            }
        }

        let (rτ, τ) = ctx.parse_top(tokens, warnings)?;
        r = InputRegionTag::max(r, rτ);
        return Ok((r, TypeTerm::Univ{
            Γ, bounds, τ: Box::new(τ)
        }));
    }

    fn parse_rung<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        if let Some((rtok, tok)) = tokens.next() {
            match tok {
                Ok(LadderTypeToken::Univ) => self.parse_univ(tokens, warnings),
                Ok(LadderTypeToken::OpenSpec) => self.parse_app(tokens, warnings),
                Ok(LadderTypeToken::OpenSeq) => self.parse_seq(tokens, warnings),
                Ok(LadderTypeToken::OpenStruct) => self.parse_struct(tokens, warnings),

                Ok(LadderTypeToken::Open) => {
                    let (rt, t) = self.parse_ladder(tokens, warnings)?;
                    match tokens.next() {
                        Some((rtok2, Ok(LadderTypeToken::Close))) => {
                            return Ok((rt, t));
                        }
                        Some((rtok2, Ok(_))) => Err((rtok2, ParseError::UnexpectedToken)),
                        Some((rtok2, Err(err))) => Err((rtok2, ParseError::LexError(err))),
                        None => Err((rt, ParseError::UnexpectedEnd))
                    }
                }

                Ok(LadderTypeToken::Close)
                | Ok(LadderTypeToken::CloseSpec)
                | Ok(LadderTypeToken::CloseSeq)
                | Ok(LadderTypeToken::CloseStruct)
                    => Err((InputRegionTag::default(), ParseError::UnexpectedClose)),

                Ok(LadderTypeToken::StructSep)
                | Ok(LadderTypeToken::EnumSep)
                | Ok(LadderTypeToken::AssignType)
                | Ok(LadderTypeToken::SubType)
                | Ok(LadderTypeToken::TraitType)
                | Ok(LadderTypeToken::ParallelType)
                | Ok(LadderTypeToken::ArrowFunc)
                | Ok(LadderTypeToken::ArrowMorph)

                => Err((rtok, ParseError::UnexpectedToken)),

                Ok(LadderTypeToken::Ladder) => Err((rtok, ParseError::UnexpectedLadder)),
                Ok(LadderTypeToken::Symbol(s)) => {

                    if self.get_typeid(&s).is_none() {
                        warnings.push(ParseInfo {
                            char_range: rtok,
                            info: ParseInfoType::UnknownTypeName(s.clone())
                        });
                    }

                    Ok((InputRegionTag::default(),
                        match self.get_typeid_creat(&s) {
                            TypeID::Fun(id) => TypeTerm::Id(id),
                            TypeID::Var(id) => TypeTerm::Var(id)
                        }
                    ))
                },
                Ok(LadderTypeToken::Char(c)) => Ok((rtok, TypeTerm::Char(c))),
                Ok(LadderTypeToken::Num(n)) => Ok((rtok, TypeTerm::Num(n))),
                Err(err) => Err((rtok, ParseError::LexError(err))),
            }
        } else {
            Err((InputRegionTag::default(), ParseError::UnexpectedEnd))
        }
    }

    fn parse_ladder<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>, warnings: &mut Vec<ParseInfo>) -> ParseLadderTypeResult
    where It: Iterator<Item = char>
    {
        let mut rungs = Vec::new();
        let mut r = InputRegionTag::default();

        match self.parse_rung(tokens, warnings) {
            Ok((rt, t)) => {
                r = InputRegionTag::max(r, rt);
                rungs.push(t);
            }
            Err((rt, err)) => { return Err((rt, err)); }
        }

        while let Some((rtok, tok)) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Ladder) => {
                    let (rtok, tok) = tokens.next().unwrap();
                    r = InputRegionTag::max(r, rtok);

                    if rungs.len() > 0 {
                        match self.parse_rung(tokens, warnings) {
                            Ok((rt, t)) => {
                                r = InputRegionTag::max(r, rt);
                                rungs.push(t);
                            }
                            Err(err) => { return Err(err); }
                        }
                    } else {
                        return Err((rtok, ParseError::UnexpectedLadder));
                    }
                }
                Err(lexerr) => {
                    return Err((*rtok, ParseError::LexError(lexerr.clone())));
                }
                _ => {
                    break;
                }
            }
        }

        match rungs.len() {
            0 => Err((r, ParseError::UnexpectedEnd)),
            1 => Ok((r, rungs[0].clone())),
            _ => Ok((r, TypeTerm::Ladder(rungs))),
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
