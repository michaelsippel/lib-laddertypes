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

        match self.parse_ladder(&mut tokens) {
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
                    match self.parse_ladder(tokens) {
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
        let mut items = Vec::new();

        if let Some(Ok(LadderTypeToken::Ladder)) = tokens.peek() {
            tokens.next();
            seq_repr = Some(Box::new(self.parse_ladder(tokens)?));
        }

        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::CloseSeq) => {
                    tokens.next();
                    return Ok(TypeTerm::Seq { seq_repr, items });
                }
                _ => {
                    match self.parse_ladder(tokens) {
                        Ok(a) => { items.push(a);  }
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
            struct_repr = Some(Box::new(self.parse_ladder(tokens)?));
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

                            let ty = self.parse_ladder(tokens)?;
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

                    let ty = self.parse_ladder(tokens)?;
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
        match tokens.next() {
            Some(Ok(LadderTypeToken::Open)) => {
                let symbol = match tokens.next() {
                    Some(Ok(LadderTypeToken::Symbol(symbol))) => {
                        symbol
                    }
                    Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                    Some(_) => { return Err(ParseError::UnexpectedToken); }
                    None => { return Err(ParseError::UnexpectedEnd); }
                };

                let var_bound = match tokens.next() {
                    Some(Ok(LadderTypeToken::AssignType)) => {
                        let t = self.parse_ladder(tokens)?;
                        VariableConstraint::ValueUInt
                    }
                    Some(Ok(LadderTypeToken::AssignSubType)) => {
                        let t = self.parse_ladder(tokens)?;
                        VariableConstraint::Subtype(t)
                    }
                    Some(Ok(LadderTypeToken::AssignTraitType)) => {
                        let t = self.parse_ladder(tokens)?;
                        VariableConstraint::Trait(t)
                    }
                    Some(Ok(LadderTypeToken::AssignParallelType)) => {
                        let t = self.parse_ladder(tokens)?;
                        VariableConstraint::Parallel(t)
                    }

                    Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                    Some(_) => { return Err(ParseError::UnexpectedToken); }
                    None => { return Err(ParseError::UnexpectedEnd); }
                };

                // `)` at end
                match tokens.next() {
                    Some(Ok(LadderTypeToken::Close)) => {
                        let mut ctx = self.scope();
                        ctx.add_variable(&symbol, match &var_bound {
                            VariableConstraint::ValueUInt => TypeKind::ValueUInt,
                            VariableConstraint::Subtype(t) => TypeKind::Type,
                            VariableConstraint::Trait(t) => TypeKind::Type,
                            VariableConstraint::Parallel(t) => TypeKind::Type,
                            VariableConstraint::UnconstrainedType => TypeKind::Type
                        });

                        let ty = ctx.parse_ladder(tokens)?;
                        // todo: return CTX here!
                        return Ok(TypeTerm::Univ(Box::new(var_bound), Box::new(ty)));
                    }
                    Some(Err(err)) => { return Err(ParseError::LexError(err)); }
                    Some(_) => { return Err(ParseError::UnexpectedToken); }
                    None => { return Err(ParseError::UnexpectedEnd); }
                }

            }
            Some(Err(err)) => { return Err(ParseError::LexError(err)); }
            Some(_) => { return Err(ParseError::UnexpectedToken); }
            None => { return Err(ParseError::UnexpectedEnd); }
        }
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
            | Some(Ok(LadderTypeToken::AssignSubType))
            | Some(Ok(LadderTypeToken::AssignTraitType))
            | Some(Ok(LadderTypeToken::AssignParallelType))
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
