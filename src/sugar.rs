use {
    crate::{parser::ParseLadderType, subtype_unify, TypeDict, TypeID, TypeTerm}
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SugaredStructMember {
    pub symbol: String,
    pub ty: SugaredTypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SugaredEnumVariant {
    pub symbol: String,
    pub ty: SugaredTypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SugaredTypeTerm {
    TypeID(TypeID),
    Num(i64),
    Char(char),
    Univ(Box< SugaredTypeTerm >),
    Spec(Vec< SugaredTypeTerm >),
    Func(Vec< SugaredTypeTerm >),
    Morph(Vec< SugaredTypeTerm >),
    Ladder(Vec< SugaredTypeTerm >),
    Struct{
        struct_repr: Option< Box<SugaredTypeTerm> >,
        members: Vec< SugaredStructMember >
    },
    Enum{
        enum_repr: Option<Box< SugaredTypeTerm >>,
        variants: Vec< SugaredEnumVariant >
    },
    Seq{
        seq_repr: Option<Box< SugaredTypeTerm >>,
        items: Vec< SugaredTypeTerm >
    },

    /*
    Todo: Ref, RefMut
    */
}

impl SugaredStructMember {
    pub fn parse( dict: &mut impl TypeDict, ty: &TypeTerm ) -> Option<Self> {
        match ty {
            TypeTerm::App(args) => {
                if args.len() != 2 {
                    return None;
                }
/*
                if args[0] != dict.parse("Struct.Field").expect("parse") {
                    return None;
                }
*/
                let symbol = match args[0] {
                    TypeTerm::Char(c) => c.to_string(),
                    TypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[1].clone().sugar(dict);

                Some(SugaredStructMember { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
}

impl SugaredEnumVariant {
    pub fn parse( dict: &mut impl TypeDict, ty: &TypeTerm ) -> Option<Self> {
        match ty {
            TypeTerm::App(args) => {
                if args.len() != 2 {
                    return None;
                }
/*
                if args[0] != dict.parse("Enum.Variant").expect("parse") {
                    return None;
                }
*/
                let symbol = match args[0] {
                    TypeTerm::Char(c) => c.to_string(),
                    TypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[1].clone().sugar(dict);

                Some(SugaredEnumVariant { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
}

impl TypeTerm {
    pub fn sugar(self: TypeTerm, dict: &mut impl crate::TypeDict) -> SugaredTypeTerm {
        dict.add_varname("StructRepr".into());
        dict.add_varname("EnumRepr".into());
        dict.add_varname("SeqRepr".into());

        match self {
            TypeTerm::TypeID(id) => SugaredTypeTerm::TypeID(id),
            TypeTerm::Num(n) => SugaredTypeTerm::Num(n),
            TypeTerm::Char(c) => SugaredTypeTerm::Char(c),
            TypeTerm::App(args) => if let Some(first) = args.first() {
                if first == &dict.parse("Func").unwrap() {
                    SugaredTypeTerm::Func( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Morph").unwrap() {
                    SugaredTypeTerm::Morph( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Struct").unwrap() {
                    SugaredTypeTerm::Struct{
                        struct_repr: None,
                        members: args[1..].into_iter()
                            .map(|t| SugaredStructMember::parse(dict, t).expect("cant parse field"))
                            .collect()
                    }
                }
                else if let Ok(σ) = crate::unify( first, &dict.parse("Struct ~ StructRepr").expect("") ) {
                    SugaredTypeTerm::Struct{
                        struct_repr: Some(Box::new(σ.get(&dict.get_typeid(&"StructRepr".into()).expect("")).unwrap().clone().sugar(dict))),
                        members: args[1..].into_iter()
                            .map(|t| SugaredStructMember::parse(dict, t).expect("cant parse field"))
                            .collect()
                    }
                }
                else if first == &dict.parse("Enum").unwrap() {
                    SugaredTypeTerm::Enum{
                        enum_repr: None,
                        variants: args[1..].into_iter()
                            .map(|t| SugaredEnumVariant::parse(dict, t).expect("cant parse variant"))
                            .collect()
                    }
                }
                else if let Ok(σ) = crate::unify( first, &dict.parse("Enum ~ EnumRepr").expect("") ) {
                    SugaredTypeTerm::Enum{
                        enum_repr: Some(Box::new(σ.get(&dict.get_typeid(&"EnumRepr".into()).expect("")).unwrap().clone().sugar(dict))),
                        variants: args[1..].into_iter()
                            .map(|t| SugaredEnumVariant::parse(dict, t).expect("cant parse variant"))
                            .collect()
                    }
                }
                else if first == &dict.parse("Seq").unwrap() {
                    SugaredTypeTerm::Seq {
                        seq_repr: None,
                        items: args[1..].into_iter()
                            .map(|t| t.clone().sugar(dict))
                            .collect()
                    }
                }
                else if let Ok(σ) = crate::unify( first, &dict.parse("Seq ~ SeqRepr").expect("") ) {
                    SugaredTypeTerm::Seq {
                        seq_repr: Some(Box::new(σ.get(&dict.get_typeid(&"SeqRepr".into()).expect("")).unwrap().clone().sugar(dict))),
                        items: args[1..].into_iter()
                            .map(|t| t.clone().sugar(dict))
                            .collect()
                    }
                }
                else if first == &dict.parse("Spec").unwrap() {
                    SugaredTypeTerm::Spec( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Univ").unwrap() {
                    SugaredTypeTerm::Univ(Box::new(
                        SugaredTypeTerm::Spec(
                            args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect()
                        )
                    ))
                }
                else {
                    SugaredTypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
                }
            } else {
                SugaredTypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
            },
            TypeTerm::Ladder(rungs) =>
               SugaredTypeTerm::Ladder(rungs.into_iter().map(|t| t.sugar(dict)).collect())
        }
    }
}


impl SugaredStructMember {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        TypeTerm::App(vec![
            //dict.parse("Struct.Field").expect("parse"),
            dict.parse(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
    }
}

impl SugaredEnumVariant {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        TypeTerm::App(vec![
            //dict.parse("Enum.Variant").expect("parse"),
            dict.parse(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
    }
}

impl SugaredTypeTerm {
    pub fn unit() -> Self {
        SugaredTypeTerm::Ladder(vec![])
    }

    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        match self {
            SugaredTypeTerm::TypeID(id) => TypeTerm::TypeID(id),
            SugaredTypeTerm::Num(n) => TypeTerm::Num(n),
            SugaredTypeTerm::Char(c) => TypeTerm::Char(c),
            SugaredTypeTerm::Univ(t) => t.desugar(dict),
            SugaredTypeTerm::Spec(ts) => TypeTerm::App(ts.into_iter().map(|t| t.desugar(dict)).collect()),
            SugaredTypeTerm::Ladder(ts) => TypeTerm::Ladder(ts.into_iter().map(|t|t.desugar(dict)).collect()),
            SugaredTypeTerm::Func(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Func").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Morph(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Morph").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Struct{ struct_repr, members } => TypeTerm::App(
                std::iter::once(
                    if let Some(sr) = struct_repr {
                        TypeTerm::Ladder(vec![
                            dict.parse("Struct").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse("Struct").unwrap()
                    }
                ).chain(
                    members.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Enum{ enum_repr, variants } => TypeTerm::App(
                std::iter::once(
                    if let Some(sr) = enum_repr {
                        TypeTerm::Ladder(vec![
                            dict.parse("Enum").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse("Enum").unwrap()
                    }
                ).chain(
                    variants.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Seq{ seq_repr, items } => TypeTerm::App(
                std::iter::once(
                    if let Some(sr) = seq_repr {
                        TypeTerm::Ladder(vec![
                            dict.parse("Seq").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse("Seq").unwrap()
                    }
                ).chain(
                    items.into_iter().map(|t| t.desugar(dict))
                ).collect()),
        }
    }

    pub fn contains_var(&self, var_id: u64) -> bool {
        match self {
            SugaredTypeTerm::TypeID(TypeID::Var(v)) => (&var_id == v),
            SugaredTypeTerm::Spec(args) |
            SugaredTypeTerm::Func(args) |
            SugaredTypeTerm::Morph(args) |
            SugaredTypeTerm::Ladder(args) => {
                for a in args.iter() {
                    if a.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            SugaredTypeTerm::Univ(t) => {
                t.contains_var(var_id)
            }
            SugaredTypeTerm::Struct { struct_repr, members } => {
                if let Some(struct_repr) =  struct_repr {
                    if struct_repr.contains_var(var_id) {
                        return true;
                    }
                }

                for SugaredStructMember{ symbol, ty } in members {
                    if ty.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            SugaredTypeTerm::Enum { enum_repr, variants } => {
                if let Some(enum_repr) =  enum_repr {
                    if enum_repr.contains_var(var_id) {
                        return true;
                    }
                }

                for SugaredEnumVariant{ symbol, ty } in variants {
                    if ty.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            SugaredTypeTerm::Seq { seq_repr, items } => {
                if let Some(seq_repr) =  seq_repr {
                    if seq_repr.contains_var(var_id) {
                        return true;
                    }
                }

                for ty in items {
                    if ty.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }

            SugaredTypeTerm::Num(_) |
            SugaredTypeTerm::Char(_) |
            SugaredTypeTerm::TypeID(TypeID::Fun(_)) => false
        }
    }

    pub fn strip(self) -> SugaredTypeTerm {
        if self.is_empty() {
            return SugaredTypeTerm::unit();
        }

        match self {
            SugaredTypeTerm::Ladder(rungs) => {
                let mut rungs :Vec<_> = rungs.into_iter()
                    .filter_map(|mut r| {
                        r = r.strip();
                        if r != SugaredTypeTerm::unit() {
                            Some(match r {
                                SugaredTypeTerm::Ladder(r) => r,
                                a => vec![ a ]
                            })
                        }
                        else { None }
                    })
                .flatten()
                .collect();

                if rungs.len() == 1 {
                    rungs.pop().unwrap()
                } else {
                    SugaredTypeTerm::Ladder(rungs)
                }
            },
            SugaredTypeTerm::Spec(args) => {
                let mut args :Vec<_> = args.into_iter().map(|arg| arg.strip()).collect();
                if args.len() == 0 {
                    SugaredTypeTerm::unit()
                } else if args.len() == 1 {
                    args.pop().unwrap()
                } else {
                    SugaredTypeTerm::Spec(args)
                }
            }
            SugaredTypeTerm::Seq{ mut seq_repr, mut items } => {
                if let Some(seq_repr) = seq_repr.as_mut() {
                    *seq_repr = Box::new(seq_repr.clone().strip());
                }
                for i in items.iter_mut() {
                    *i = i.clone().strip();
                }

                SugaredTypeTerm::Seq { seq_repr, items }
            }
            atom => atom
        }
    }


    pub fn get_interface_type(&self) -> SugaredTypeTerm {
        match self {
            SugaredTypeTerm::Ladder(rungs) => {
                if let Some(top) = rungs.first() {
                    top.get_interface_type()
                } else {
                    SugaredTypeTerm::unit()
                }
            }
            SugaredTypeTerm::Spec(args)
                => SugaredTypeTerm::Spec(args.iter().map(|a| a.get_interface_type()).collect()),

            SugaredTypeTerm::Func(args)
                => SugaredTypeTerm::Func(args.iter().map(|a| a.get_interface_type()).collect()),

            SugaredTypeTerm::Morph(args)
                => SugaredTypeTerm::Spec(args.iter().map(|a| a.get_interface_type()).collect()),

            SugaredTypeTerm::Univ(t)
                => SugaredTypeTerm::Univ(Box::new(t.get_interface_type())),

            SugaredTypeTerm::Seq { seq_repr, items } => {
                SugaredTypeTerm::Seq {
                    seq_repr: if let Some(sr) = seq_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    items: items.iter().map(|t| t.get_interface_type()).collect()
                }
            }
            SugaredTypeTerm::Struct { struct_repr, members } => {
                SugaredTypeTerm::Struct {
                    struct_repr: if let Some(sr) = struct_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    members: members.iter()
                        .map(|SugaredStructMember{symbol,ty}|
                            SugaredStructMember {symbol:symbol.clone(), ty:ty.get_interface_type() })
                        .collect()
                }
            }
            SugaredTypeTerm::Enum { enum_repr, variants } => {
                SugaredTypeTerm::Enum {
                    enum_repr: if let Some(sr) = enum_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    variants: variants.iter()
                        .map(|SugaredEnumVariant{symbol,ty}|
                            SugaredEnumVariant{ symbol:symbol.clone(), ty:ty.get_interface_type() })
                        .collect()
                }
            }

            SugaredTypeTerm::TypeID(tyid) => SugaredTypeTerm::TypeID(tyid.clone()),
            SugaredTypeTerm::Num(n) => SugaredTypeTerm::Num(*n),
            SugaredTypeTerm::Char(c) => SugaredTypeTerm::Char(*c)
        }
    }

    pub fn get_floor_type(&self) -> (SugaredTypeTerm, SugaredTypeTerm) {
        match self.clone() {
            SugaredTypeTerm::Ladder(mut rungs) => {
                if let Some(bot) = rungs.pop() {
                    let (bot_ψ, bot_floor) = bot.get_floor_type();
                    rungs.push(bot_ψ);
                    (SugaredTypeTerm::Ladder(rungs).strip(), bot_floor.strip())
                } else {
                    (SugaredTypeTerm::unit(), SugaredTypeTerm::unit())
                }
            }
            /*
            SugaredTypeTerm::Spec(args)
                => (SugaredTypeTerm::SugaredTypeTerm::Spec(args.iter().map(|a| a.get_floor_type()).collect()),

            SugaredTypeTerm::Func(args)
                => SugaredTypeTerm::Func(args.iter().map(|a| a.get_floor_type()).collect()),

            SugaredTypeTerm::Morph(args)
                => SugaredTypeTerm::Spec(args.iter().map(|a| a.get_floor_type()).collect()),

            SugaredTypeTerm::Univ(t)
                => SugaredTypeTerm::Univ(Box::new(t.get_floor_type())),

            SugaredTypeTerm::Seq { seq_repr, items } => {
                SugaredTypeTerm::Seq {
                    seq_repr: if let Some(sr) = seq_repr {
                        Some(Box::new(sr.clone().get_floor_type()))
                    } else { None },
                    items: items.iter().map(|t| t.get_floor_type()).collect()
                }
            }
            SugaredTypeTerm::Struct { struct_repr, members } => {
                SugaredTypeTerm::Struct {
                    struct_repr: if let Some(sr) = struct_repr {
                        Some(Box::new(sr.clone().get_floor_type()))
                    } else { None },
                    members: members.iter()
                        .map(|SugaredStructMember{symbol,ty}|
                            SugaredStructMember {symbol:symbol.clone(), ty:ty.get_floor_type() })
                        .collect()
                }
            }
            SugaredTypeTerm::Enum { enum_repr, variants } => {
                SugaredTypeTerm::Enum {
                    enum_repr: if let Some(sr) = enum_repr {
                        Some(Box::new(sr.clone().get_floor_type()))
                    } else { None },
                    variants: variants.iter()
                        .map(|SugaredEnumVariant{symbol,ty}|
                            SugaredEnumVariant{ symbol:symbol.clone(), ty:ty.get_floor_type() })
                        .collect()
                }
            }

            SugaredTypeTerm::TypeID(tyid) => SugaredTypeTerm::TypeID(tyid.clone()),
            SugaredTypeTerm::Num(n) => SugaredTypeTerm::Num(*n),
            SugaredTypeTerm::Char(c) => SugaredTypeTerm::Char(*c)
            */

            other => (SugaredTypeTerm::unit(), other.clone().strip())
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            SugaredTypeTerm::TypeID(_) => false,
            SugaredTypeTerm::Num(_) => false,
            SugaredTypeTerm::Char(_) => false,
            SugaredTypeTerm::Univ(t) => t.is_empty(),
            SugaredTypeTerm::Spec(ts) |
            SugaredTypeTerm::Ladder(ts) |
            SugaredTypeTerm::Func(ts) |
            SugaredTypeTerm::Morph(ts) => {
                ts.iter().fold(true, |s,t| s && t.is_empty() )
            }
            SugaredTypeTerm::Seq{ seq_repr, items } => {
                items.iter().fold(true, |s,t| s && t.is_empty() )
            }
            SugaredTypeTerm::Struct{ struct_repr, members } => {
                members.iter()
                    .fold(true, |s,member_decl| s && member_decl.ty.is_empty() )
            }
            SugaredTypeTerm::Enum{ enum_repr, variants } => {
                variants.iter()
                    .fold(true, |s,variant_decl| s && variant_decl.ty.is_empty() )
            }
        }
    }
}
