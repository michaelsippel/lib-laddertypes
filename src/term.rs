use {
    crate::{parser::ParseLadderType, subtype_unify, DesugaredTypeTerm, TypeDict, TypeID}
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructMember {
    pub symbol: String,
    pub ty: TypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnumVariant {
    pub symbol: String,
    pub ty: TypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeTerm {
    TypeID(TypeID),
    Num(i64),
    Char(char),
    Univ(Box< TypeTerm >),
    Spec(Vec< TypeTerm >),
    Func(Vec< TypeTerm >),
    Morph(Vec< TypeTerm >),
    Ladder(Vec< TypeTerm >),
    Struct{
        struct_repr: Option< Box<TypeTerm> >,
        members: Vec< StructMember >
    },
    Enum{
        enum_repr: Option<Box< TypeTerm >>,
        variants: Vec< EnumVariant >
    },
    Seq{
        seq_repr: Option<Box< TypeTerm >>,
        items: Vec< TypeTerm >
    },

    /*
    Todo: Ref, RefMut
    */
}

impl StructMember {
    pub fn parse( dict: &mut impl TypeDict, ty: &DesugaredTypeTerm ) -> Option<Self> {
        match ty {
            DesugaredTypeTerm::App(args) => {
                if args.len() != 2 {
                    return None;
                }
/*
                if args[0] != dict.parse("Struct.Field").expect("parse") {
                    return None;
                }
*/
                let symbol = match args[0] {
                    DesugaredTypeTerm::Char(c) => c.to_string(),
                    DesugaredTypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[1].clone().sugar(dict);

                Some(StructMember { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
}

impl EnumVariant {
    pub fn parse( dict: &mut impl TypeDict, ty: &DesugaredTypeTerm ) -> Option<Self> {
        match ty {
            DesugaredTypeTerm::App(args) => {
                if args.len() != 2 {
                    return None;
                }
/*
                if args[0] != dict.parse("Enum.Variant").expect("parse") {
                    return None;
                }
*/
                let symbol = match args[0] {
                    DesugaredTypeTerm::Char(c) => c.to_string(),
                    DesugaredTypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[1].clone().sugar(dict);

                Some(EnumVariant { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
}

impl DesugaredTypeTerm {
    pub fn sugar(self: DesugaredTypeTerm, dict: &mut impl crate::TypeDict) -> TypeTerm {
        dict.add_varname("StructRepr".into());
        dict.add_varname("EnumRepr".into());
        dict.add_varname("SeqRepr".into());

        match self {
            DesugaredTypeTerm::TypeID(id) => TypeTerm::TypeID(id),
            DesugaredTypeTerm::Num(n) => TypeTerm::Num(n),
            DesugaredTypeTerm::Char(c) => TypeTerm::Char(c),
            DesugaredTypeTerm::App(args) => if let Some(first) = args.first() {
                if first == &dict.parse_desugared("Func").unwrap() {
                    TypeTerm::Func( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse_desugared("Morph").unwrap() {
                    TypeTerm::Morph( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse_desugared("Seq").unwrap() {
                    TypeTerm::Seq{
                        seq_repr: None,
                        items: args[1..].into_iter()
                            .map(|t| t.clone().sugar(dict))
                            .collect()
                    }
                }
                else if first == &dict.parse_desugared("Struct").unwrap() {
                    TypeTerm::Struct{
                        struct_repr: None,
                        members: args[1..].into_iter()
                            .map(|t| StructMember::parse(dict, t).expect("cant parse field"))
                            .collect()
                    }
                }
                else if first == &dict.parse_desugared("Enum").unwrap() {
                    TypeTerm::Enum{
                        enum_repr: None,
                        variants: args[1..].into_iter()
                            .map(|t| EnumVariant::parse(dict, t).expect("cant parse variant"))
                            .collect()
                    }
                }
                else if let DesugaredTypeTerm::Ladder(mut rungs) = first.clone() {
                    if rungs.len() > 0 {
                        match rungs.remove(0) {
                            DesugaredTypeTerm::TypeID(tyid) => {
                                if tyid == dict.get_typeid(&"Seq".into()).expect("") {
                                    TypeTerm::Seq {
                                        seq_repr:
                                            if rungs.len() > 0 {
                                                Some(Box::new(
                                                    TypeTerm::Ladder(rungs.into_iter()
                                                        .map(|r| r.clone().sugar(dict))
                                                        .collect()
                                                    ).normalize()
                                                ))
                                            } else {
                                                None
                                            },
                                        items: args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect()
                                    }
                                } else if tyid == dict.get_typeid(&"Struct".into()).expect("") {
                                    TypeTerm::Struct {
                                        struct_repr:
                                            if rungs.len() > 0 {
                                                Some(Box::new(
                                                    TypeTerm::Ladder(rungs.into_iter()
                                                        .map(|r| r.clone().sugar(dict))
                                                        .collect()
                                                    ).normalize()
                                                ))
                                            } else {
                                                None
                                            },
                                        members: args[1..].into_iter()
                                            .map(|t| StructMember::parse(dict, t).expect("cant parse field"))
                                            .collect()
                                    }
                                } else if tyid == dict.get_typeid(&"Enum".into()).expect("") {
                                    TypeTerm::Enum {
                                        enum_repr:
                                            if rungs.len() > 0 {
                                                Some(Box::new(
                                                    TypeTerm::Ladder(rungs.into_iter()
                                                        .map(|r| r.clone().sugar(dict))
                                                        .collect()
                                                    ).normalize()
                                                ))
                                            } else {
                                                None
                                            },
                                        variants: args[1..].into_iter()
                                            .map(|t| EnumVariant::parse(dict, t).expect("cant parse field"))
                                            .collect()
                                    }
                                } else {
                                    TypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
                                }
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }

                else if first == &dict.parse_desugared("Spec").unwrap() {
                    TypeTerm::Spec( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse_desugared("Univ").unwrap() {
                    TypeTerm::Univ(Box::new(
                        TypeTerm::Spec(
                            args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect()
                        )
                    ))
                }
                else {
                    TypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
                }
            } else {
                TypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
            },
            DesugaredTypeTerm::Ladder(rungs) =>
               TypeTerm::Ladder(rungs.into_iter().map(|t| t.sugar(dict)).collect())
        }
    }
}


impl StructMember {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> DesugaredTypeTerm {
        DesugaredTypeTerm::App(vec![
            //dict.parse("Struct.Field").expect("parse"),
            dict.parse_desugared(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
    }
}

impl EnumVariant {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> DesugaredTypeTerm {
        DesugaredTypeTerm::App(vec![
            //dict.parse("Enum.Variant").expect("parse"),
            dict.parse_desugared(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
    }
}

impl TypeTerm {
    pub fn unit() -> Self {
        TypeTerm::Ladder(vec![])
    }

    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> DesugaredTypeTerm {
        match self {
            TypeTerm::TypeID(id) => DesugaredTypeTerm::TypeID(id),
            TypeTerm::Num(n) => DesugaredTypeTerm::Num(n),
            TypeTerm::Char(c) => DesugaredTypeTerm::Char(c),
            TypeTerm::Univ(t) => t.desugar(dict),
            TypeTerm::Spec(ts) => DesugaredTypeTerm::App(ts.into_iter().map(|t| t.desugar(dict)).collect()),
            TypeTerm::Ladder(ts) => DesugaredTypeTerm::Ladder(ts.into_iter().map(|t|t.desugar(dict)).collect()),
            TypeTerm::Func(ts) => DesugaredTypeTerm::App(
                std::iter::once( dict.parse_desugared("Func").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            TypeTerm::Morph(ts) => DesugaredTypeTerm::App(
                std::iter::once( dict.parse_desugared("Morph").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            TypeTerm::Struct{ struct_repr, members } => DesugaredTypeTerm::App(
                std::iter::once(
                    if let Some(sr) = struct_repr {
                        DesugaredTypeTerm::Ladder(vec![
                            dict.parse_desugared("Struct").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse_desugared("Struct").unwrap()
                    }
                ).chain(
                    members.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            TypeTerm::Enum{ enum_repr, variants } => DesugaredTypeTerm::App(
                std::iter::once(
                    if let Some(sr) = enum_repr {
                        DesugaredTypeTerm::Ladder(vec![
                            dict.parse_desugared("Enum").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse_desugared("Enum").unwrap()
                    }
                ).chain(
                    variants.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            TypeTerm::Seq{ seq_repr, items } => DesugaredTypeTerm::App(
                std::iter::once(
                    if let Some(sr) = seq_repr {
                        DesugaredTypeTerm::Ladder(vec![
                            dict.parse_desugared("Seq").unwrap(),
                            sr.desugar(dict)
                        ])
                    } else {
                        dict.parse_desugared("Seq").unwrap()
                    }
                ).chain(
                    items.into_iter().map(|t| t.desugar(dict))
                ).collect()),
        }
    }

    pub fn contains_var(&self, var_id: u64) -> bool {
        match self {
            TypeTerm::TypeID(TypeID::Var(v)) => (&var_id == v),
            TypeTerm::Spec(args) |
            TypeTerm::Func(args) |
            TypeTerm::Morph(args) |
            TypeTerm::Ladder(args) => {
                for a in args.iter() {
                    if a.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            TypeTerm::Univ(t) => {
                t.contains_var(var_id)
            }
            TypeTerm::Struct { struct_repr, members } => {
                if let Some(struct_repr) =  struct_repr {
                    if struct_repr.contains_var(var_id) {
                        return true;
                    }
                }

                for StructMember{ symbol, ty } in members {
                    if ty.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            TypeTerm::Enum { enum_repr, variants } => {
                if let Some(enum_repr) =  enum_repr {
                    if enum_repr.contains_var(var_id) {
                        return true;
                    }
                }

                for EnumVariant{ symbol, ty } in variants {
                    if ty.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            TypeTerm::Seq { seq_repr, items } => {
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

            TypeTerm::Num(_) |
            TypeTerm::Char(_) |
            TypeTerm::TypeID(TypeID::Fun(_)) => false
        }
    }

    pub fn strip(self) -> TypeTerm {
        if self.is_empty() {
            return TypeTerm::unit();
        }

        match self {
            TypeTerm::Ladder(rungs) => {
                let mut rungs :Vec<_> = rungs.into_iter()
                    .filter_map(|mut r| {
                        r = r.strip();
                        if r != TypeTerm::unit() {
                            Some(match r {
                                TypeTerm::Ladder(r) => r,
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
                    TypeTerm::Ladder(rungs)
                }
            },
            TypeTerm::Spec(args) => {
                let mut args :Vec<_> = args.into_iter().map(|arg| arg.strip()).collect();
                if args.len() == 0 {
                    TypeTerm::unit()
                } else if args.len() == 1 {
                    args.pop().unwrap()
                } else {
                    TypeTerm::Spec(args)
                }
            }
            TypeTerm::Seq{ mut seq_repr, mut items } => {
                if let Some(seq_repr) = seq_repr.as_mut() {
                    *seq_repr = Box::new(seq_repr.clone().strip());
                }
                for i in items.iter_mut() {
                    *i = i.clone().strip();
                }

                TypeTerm::Seq { seq_repr, items }
            }
            atom => atom
        }
    }

    pub fn get_interface_type(&self) -> TypeTerm {
        match self {
            TypeTerm::Ladder(rungs) => {
                if let Some(top) = rungs.first() {
                    top.get_interface_type()
                } else {
                    TypeTerm::unit()
                }
            }
            TypeTerm::Spec(args)
                => TypeTerm::Spec(args.iter().map(|a| a.get_interface_type()).collect()),

            TypeTerm::Func(args)
                => TypeTerm::Func(args.iter().map(|a| a.get_interface_type()).collect()),

            TypeTerm::Morph(args)
                => TypeTerm::Spec(args.iter().map(|a| a.get_interface_type()).collect()),

            TypeTerm::Univ(t)
                => TypeTerm::Univ(Box::new(t.get_interface_type())),

            TypeTerm::Seq { seq_repr, items } => {
                TypeTerm::Seq {
                    seq_repr: if let Some(sr) = seq_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    items: items.iter().map(|t| t.get_interface_type()).collect()
                }
            }
            TypeTerm::Struct { struct_repr, members } => {
                TypeTerm::Struct {
                    struct_repr: if let Some(sr) = struct_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    members: members.iter()
                        .map(|StructMember{symbol,ty}|
                            StructMember {symbol:symbol.clone(), ty:ty.get_interface_type() })
                        .collect()
                }
            }
            TypeTerm::Enum { enum_repr, variants } => {
                TypeTerm::Enum {
                    enum_repr: if let Some(sr) = enum_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    variants: variants.iter()
                        .map(|EnumVariant{symbol,ty}|
                            EnumVariant{ symbol:symbol.clone(), ty:ty.get_interface_type() })
                        .collect()
                }
            }

            TypeTerm::TypeID(tyid) => TypeTerm::TypeID(tyid.clone()),
            TypeTerm::Num(n) => TypeTerm::Num(*n),
            TypeTerm::Char(c) => TypeTerm::Char(*c)
        }
    }

    pub fn get_floor_type(&self) -> (TypeTerm, TypeTerm) {
        match self.clone() {
            TypeTerm::Ladder(mut rungs) => {
                if let Some(bot) = rungs.pop() {
                    let (bot_ψ, bot_floor) = bot.get_floor_type();
                    rungs.push(bot_ψ);
                    (TypeTerm::Ladder(rungs).strip(), bot_floor.strip())
                } else {
                    (TypeTerm::unit(), TypeTerm::unit())
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

            other => (TypeTerm::unit(), other.clone().strip())
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            TypeTerm::TypeID(_) => false,
            TypeTerm::Num(_) => false,
            TypeTerm::Char(_) => false,
            TypeTerm::Univ(t) => t.is_empty(),
            TypeTerm::Spec(ts) |
            TypeTerm::Ladder(ts) |
            TypeTerm::Func(ts) |
            TypeTerm::Morph(ts) => {
                ts.iter().fold(true, |s,t| s && t.is_empty() )
            }
            TypeTerm::Seq{ seq_repr, items } => {
                items.iter().fold(true, |s,t| s && t.is_empty() )
            }
            TypeTerm::Struct{ struct_repr, members } => {
                members.iter()
                    .fold(true, |s,member_decl| s && member_decl.ty.is_empty() )
            }
            TypeTerm::Enum{ enum_repr, variants } => {
                variants.iter()
                    .fold(true, |s,variant_decl| s && variant_decl.ty.is_empty() )
            }
        }
    }
}
