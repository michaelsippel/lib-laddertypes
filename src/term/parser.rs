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
    }, std::iter::Peekable
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedClose,
    UnexpectedLadder,
    UnexpectedEnd,
    UnexpectedToken
}

pub trait ParseLadderType {
    fn parse(&mut self, s:&str) -> Result<TypeTerm, ParseError>;

    fn parse_top<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_app<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_rung<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_ladder<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_seq<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_struct<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;

    fn parse_univ<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<T: LayeredContext> ParseLadderType for T {
    fn parse(&mut self, s: &str) -> Result<TypeTerm, ParseError> {
        let mut tokens = LadderTypeLexer::from(s.chars()).peekable();

        match self.parse_top(&mut tokens) {
            Ok(t) => {
                if let Some(_tok) = tokens.peek() {
                    Err(ParseError::UnexpectedToken)
                } else {
                    Ok(t)
                }
            }
            Err(err) => Err(err)
        }
    }

    fn parse_top<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        // 1. Ladders
        let t1 = self.parse_ladder(tokens)?;

        // 2. Arrows
        match tokens.peek() {
            Some(Ok(LadderTypeToken::ArrowFunc)) => {
                tokens.next();
                let t2 = self.parse_top(tokens)?;
                return Ok(TypeTerm::Func(vec![
                    t1, t2
                ]));
            }
            Some(Ok(LadderTypeToken::ArrowMorph)) => {
                tokens.next();
                let t2 = self.parse_top(tokens)?;
                return Ok(TypeTerm::Morph(Box::new(t1), Box::new(t2)));
            }
            Some(Err(err)) => {
                return Err(ParseError::LexError(err.clone()));
            }
            _ => {
                return Ok(t1);
            }
        }
    }

    fn parse_app<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut args = Vec::new();
        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::CloseSpec) => {
                    tokens.next();
                    return Ok(TypeTerm::Spec(args));
                }
                _ => {
                    match self.parse_top(tokens) {
                        Ok(a) => { args.push(a); }
                        Err(err) => { return Err(err); }
                    }
                }
            }
        }
        Err(ParseError::UnexpectedEnd)
    }

    fn parse_seq<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut seq_repr = None;

        if let Some(Ok(LadderTypeToken::Ladder)) = tokens.peek() {
            tokens.next();
            seq_repr = Some(Box::new(self.parse_top(tokens)?));
        }

        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::CloseSeq) => {
                    return Err(ParseError::UnexpectedClose);
                }
                _ => {
                    match self.parse_top(tokens) {
                        Ok(item) => {
                            while let Some(tok) = tokens.next() {
                                match tok {
                                    Ok(LadderTypeToken::CloseSeq) => { return Ok(TypeTerm::Seq { seq_repr, item: Box::new(item.clone()) }); }
                                    Ok(_) => { return Err(ParseError::UnexpectedToken); }
                                    Err(err)=> { return Err(ParseError::LexError(err)); }
                                }
                            }
                        }
                        Err(err) => { return Err(err); }
                    }
                }
            }
        }
        Err(ParseError::UnexpectedEnd)
    }

    fn parse_struct<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut struct_repr = None;
        let mut is_enum = false;
        let mut variants = Vec::new();
        let mut members = Vec::new();

        if let Some(Ok(LadderTypeToken::Ladder)) = tokens.peek() {
            tokens.next();
            struct_repr = Some(Box::new(self.parse_top(tokens)?));
        }

        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::CloseStruct) => {
                    tokens.next();
                    if is_enum {
                        return Ok(TypeTerm::Enum { enum_repr: struct_repr, variants })
                    } else {
                        return Ok(TypeTerm::Struct { struct_repr, members });
                    }
                }

                Ok(LadderTypeToken::EnumSep) => {
                    tokens.next();
                    is_enum = true;

                    match tokens.next() {
                        Some(Ok(LadderTypeToken::Symbol(symbol))) => {
                            let symbol = symbol.clone();

                            // `:` between identifier and type
                            match tokens.next() {
                                Some(Ok(LadderTypeToken::AssignType)) => {}
                                Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                                Some(_) => { return Err(ParseError::UnexpectedToken); }
                                None => { return Err(ParseError::UnexpectedEnd); }
                            }

                            let ty = self.parse_top(tokens)?;
                            variants.push(EnumVariant { symbol, ty });
                        }

                        Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                        Some(_) => { return Err(ParseError::UnexpectedToken); }
                        None => { return Err(ParseError::UnexpectedEnd); }
                    }
                }
                Ok(LadderTypeToken::Symbol(symbol)) => {
                    if is_enum {
                        return Err(ParseError::UnexpectedToken);
                    }

                    let symbol = symbol.clone();
                    tokens.next();

                    // `:` between identifier and type
                    match tokens.next() {
                        Some(Ok(LadderTypeToken::AssignType)) => {}
                        Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                        Some(_) => { return Err(ParseError::UnexpectedToken); }
                        None => { return Err(ParseError::UnexpectedEnd); }
                    }

                    let ty = self.parse_top(tokens)?;
                    members.push(StructMember { symbol, ty });

                    // `;` at end
                    match tokens.next() {
                        Some(Ok(LadderTypeToken::StructSep)) => {}
                        Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                        Some(_) => { return Err(ParseError::UnexpectedToken); }
                        None => { return Err(ParseError::UnexpectedEnd); }
                    }
                }

                Ok(_) => { return Err(ParseError::UnexpectedToken); }
                Err(err) => { return Err(ParseError::LexError(err.clone())); }
            }
        }

        Err(ParseError::UnexpectedEnd)
    }

    fn parse_univ<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut Γ = Vec::new();
        let mut ctx = self.scope();
        let mut bounds = Vec::new();

        // at least one symbol name follows
        while let Some(tok) = tokens.next() {
            match tok {
                Ok(LadderTypeToken::Symbol(symbol)) => {
                    match tokens.peek() {
                        Some(Ok(LadderTypeToken::AssignType)) => {
                            tokens.next();
                            let t = ctx.parse_top(tokens)?;
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Value(t.clone()) });
                            ctx.add_variable( &symbol, TypeKind::Value(t) );

                            match tokens.peek() {
                                Some(Ok(LadderTypeToken::Univ)) => {
                                    tokens.next();
                                    continue;
                                }
                                _ => { break; }
                            }
                        }
                        Some(Ok(LadderTypeToken::Univ)) => {
                            tokens.next();
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Type });
                            ctx.add_variable( &symbol, TypeKind::Type );
                            continue;
                        }
                        Some(Ok(_)) => {
                            Γ.push(ContextEntry { symbol: symbol.clone(), kind: TypeKind::Type });
                            ctx.add_variable(&symbol, TypeKind::Type );
                            break; }
                        Some(Err(err)) => { return Err(ParseError::LexError(err.clone())); }
                        None => {
                            break;
                        }
                    }
                }
                Ok(_) => {
                    return Err(ParseError::UnexpectedToken);
                }
                Err(err) => { return Err(ParseError::LexError(err.clone())); }
            }
        }

        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Open) => {
                    // another constraint
                    tokens.next(); // take opening
                    let lhs = ctx.parse_top(tokens)?;
                    let rel_token = tokens.next();
                    let rhs = ctx.parse_top(tokens)?;

                    match rel_token {
                        Some(Ok(LadderTypeToken::SubType)) => {
                            bounds.push(ConstraintPair::Subtype(lhs, rhs));
                        }
                        Some(Ok(LadderTypeToken::TraitType)) => {
                            bounds.push(ConstraintPair::Trait(lhs, rhs));
                        }
                        Some(Ok(LadderTypeToken::ParallelType)) => {
                            bounds.push(ConstraintPair::Parallel(lhs, rhs));
                        }
                        _ => {
                            todo!()
                        }
                    }

                    match tokens.next() {
                        Some(Ok(LadderTypeToken::Close)) => {
                            continue;
                        }
                        Some(Ok(_)) => { return Err(ParseError::UnexpectedToken); }
                        Some(Err(err)) => { return Err(ParseError::LexError(err.clone())) }
                        None => { return Err(ParseError::UnexpectedEnd); }
                    }

                }

                Ok(_) => {
                    break;
                }
                Err(err) => {
                    return Err(ParseError::LexError(err.clone()));
                }
            }
        }

        let τ = ctx.parse_top(tokens)?;
        return Ok(TypeTerm::Univ{
            Γ, bounds, τ: Box::new(τ)
        });
    }

    fn parse_rung<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        match tokens.next() {
            Some(Ok(LadderTypeToken::Univ)) => self.parse_univ(tokens),
            Some(Ok(LadderTypeToken::OpenSpec)) => self.parse_app(tokens),
            Some(Ok(LadderTypeToken::OpenSeq)) => self.parse_seq(tokens),
            Some(Ok(LadderTypeToken::OpenStruct)) => self.parse_struct(tokens),

            Some(Ok(LadderTypeToken::Open)) => {
                let t = self.parse_ladder(tokens)?;

                match tokens.next() {
                    Some(Ok(LadderTypeToken::Close)) => {
                        return Ok(t);
                    }
                    Some(Ok(_)) => Err(ParseError::UnexpectedToken),
                    Some(Err(err)) => Err(ParseError::LexError(err)),
                    None => Err(ParseError::UnexpectedEnd)
                }
            }

            Some(Ok(LadderTypeToken::Close))
            | Some(Ok(LadderTypeToken::CloseSpec))
            | Some(Ok(LadderTypeToken::CloseSeq))
            | Some(Ok(LadderTypeToken::CloseStruct))
                => Err(ParseError::UnexpectedClose),

            Some(Ok(LadderTypeToken::StructSep))
            | Some(Ok(LadderTypeToken::EnumSep))
            | Some(Ok(LadderTypeToken::AssignType))
            | Some(Ok(LadderTypeToken::SubType))
            | Some(Ok(LadderTypeToken::TraitType))
            | Some(Ok(LadderTypeToken::ParallelType))
            | Some(Ok(LadderTypeToken::ArrowFunc))
            | Some(Ok(LadderTypeToken::ArrowMorph))

            => Err(ParseError::UnexpectedToken),

            Some(Ok(LadderTypeToken::Ladder)) => Err(ParseError::UnexpectedLadder),
            Some(Ok(LadderTypeToken::Symbol(s))) =>
                Ok(match self.get_typeid_creat(&s) {
                    TypeID::Fun(id) => TypeTerm::Id(id),
                    TypeID::Var(id) => TypeTerm::Var(id)
                }),
            Some(Ok(LadderTypeToken::Char(c))) => Ok(TypeTerm::Char(c)),
            Some(Ok(LadderTypeToken::Num(n))) => Ok(TypeTerm::Num(n)),
            Some(Err(err)) => Err(ParseError::LexError(err)),
            None => Err(ParseError::UnexpectedEnd)
        }
    }

    fn parse_ladder<It>(&mut self, tokens: &mut Peekable<LadderTypeLexer<It>>) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut rungs = Vec::new();

        match self.parse_rung(tokens) {
            Ok(t) => { rungs.push(t); }
            Err(err) => { return Err(err); }
        }

        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Ladder) => {
                    tokens.next();

                    if rungs.len() > 0 {
                        match self.parse_rung(tokens) {
                            Ok(t) => { rungs.push(t); }
                            Err(err) => { return Err(err); }
                        }
                    } else {
                        return Err(ParseError::UnexpectedLadder);
                    }
                }
                Err(lexerr) => {
                    return Err(ParseError::LexError(lexerr.clone()));
                }
                _ => {
                    break;
                }
            }
        }

        match rungs.len() {
            0 => Err(ParseError::UnexpectedEnd),
            1 => Ok(rungs[0].clone()),
            _ => Ok(TypeTerm::Ladder(rungs)),
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
