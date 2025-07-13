pub mod lexer;
pub mod parser;
pub mod curry;
pub mod unparser;
pub mod pnf;

#[cfg(feature = "pretty")]
mod pretty;


use {
    crate::{
        parser::ParseLadderType, ConstraintPair, ContextEntry, MorphismType, Substitution, TypeDict, TypeID, CP2},
    std::ops::Deref
};


#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum VariableConstraint {
    UnconstrainedType, // <<- add TypeKind here ?
    Subtype(TypeTerm),
    Trait(TypeTerm),
    Parallel(TypeTerm),
    ValueOf(TypeTerm),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct StructMember {
    pub symbol: String,
    pub ty: TypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct EnumVariant {
    pub symbol: String,
    pub ty: TypeTerm
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeTerm {
    Id(u64),
    Var(u64),
    Num(i64),
    Char(char),
    Univ{
        Γ: Vec< ContextEntry >,
        bounds: Vec< ConstraintPair >,
        τ: Box< TypeTerm >
    },
    Spec(Vec< TypeTerm >),
    Func(Vec< TypeTerm >),
    Morph(Box< TypeTerm >, Box< TypeTerm >),
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
        item: Box<TypeTerm>
    },

    /*
    Todo: Ref, RefMut
    */
}

impl TypeTerm {
    pub fn into_morphism_type(self) -> Option< MorphismType > {
        match self.normalize() {
            TypeTerm::Univ{ Γ, bounds, τ } => {
                let mut m = τ.into_morphism_type()?;
                m.Γ = Γ;
                m.bounds = bounds;
                Some(m)
            }
            TypeTerm::Morph(src,dst) => {
                Some(MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: src.deref().clone(),
                    dst_type: dst.deref().clone()
                })
            },
            TypeTerm::Func(args) => {
                Some(MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: args[0].clone(),
                    dst_type: args[1].clone()
                })
            }
            _ => None
        }
    }
}

impl VariableConstraint {
    pub fn normalize(&self) -> Self {
        match self {
            VariableConstraint::UnconstrainedType => VariableConstraint::UnconstrainedType,
            VariableConstraint::Subtype(τ) => VariableConstraint::Subtype(τ.clone().normalize()),
            VariableConstraint::Trait(τ) => VariableConstraint::Trait(τ.clone().normalize()),
            VariableConstraint::Parallel(τ) => VariableConstraint::Parallel(τ.clone().normalize()),
            VariableConstraint::ValueOf(τ) => VariableConstraint::ValueOf(τ.clone().normalize())
        }
    }

    pub fn apply_subst(&mut self, σ: &impl Substitution) -> &mut Self {
        match self {
            VariableConstraint::Subtype(type_term) => { type_term.apply_subst(σ); },
            VariableConstraint::Trait(type_term) => { type_term.apply_subst(σ); },
            VariableConstraint::Parallel(type_term) => { type_term.clone().apply_subst(σ); },
            _ => {}
        }
        self
    }
}

impl TypeTerm {
    pub fn unit() -> Self {
        TypeTerm::Ladder(vec![])
    }

    pub fn contains_var(&self, var_id: u64) -> bool {
        match self {
            TypeTerm::Var(v) => &var_id == v,
            TypeTerm::Spec(args) |
            TypeTerm::Func(args) |
            TypeTerm::Ladder(args) => {
                for a in args.iter() {
                    if a.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            TypeTerm::Morph(src,dst) => {
                src.contains_var(var_id) || dst.contains_var(var_id)
            }
            TypeTerm::Univ{ Γ, bounds, τ } => {
                // todo: capture avoidance (via debruijn)
                τ.contains_var( var_id + Γ.len() as u64 )
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
            TypeTerm::Seq { seq_repr, item } => {
                if let Some(seq_repr) =  seq_repr {
                    if seq_repr.contains_var(var_id) {
                        return true;
                    }
                }

                item.contains_var(var_id)
            }

            TypeTerm::Num(_) |
            TypeTerm::Char(_) |
            TypeTerm::Id(_) => false
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

            TypeTerm::Func(args) => TypeTerm::Func(args.into_iter().map(|arg| arg.strip()).collect()),
            TypeTerm::Morph(src, dst) => TypeTerm::Morph(Box::new(src.strip()), Box::new(dst.strip())),

            TypeTerm::Seq{ mut seq_repr, mut item } => {
                if let Some(seq_repr) = seq_repr.as_mut() {
                    *seq_repr = Box::new(seq_repr.clone().strip());
                }
                *item = item.clone().strip();

                TypeTerm::Seq { seq_repr, item }
            }
            TypeTerm::Struct { mut struct_repr, mut members } => {
                if let Some(struct_repr) = struct_repr.as_mut() {
                    *struct_repr = Box::new(struct_repr.clone().strip());
                }
                for m in members.iter_mut() {
                    m.ty = m.ty.clone().strip();
                }

                TypeTerm::Struct { struct_repr, members }
            },
            TypeTerm::Enum { mut enum_repr, mut variants } => {
                if let Some(enum_repr) = enum_repr.as_mut() {
                    *enum_repr = Box::new(enum_repr.clone().strip());
                }
                for v in variants.iter_mut() {
                    v.ty = v.ty.clone().strip();
                }

                TypeTerm::Enum { enum_repr, variants }
            },

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

            TypeTerm::Morph(src,dst)
                => TypeTerm::Morph(
                    Box::new(src.get_interface_type()),
                    Box::new(dst.get_interface_type())
                ),

            TypeTerm::Univ{ Γ, bounds, τ }
                => TypeTerm::Univ{
                    Γ:Γ.clone(),
                    bounds:bounds.clone(),
                    τ: Box::new(τ.get_interface_type())
                },

            TypeTerm::Seq { seq_repr, item } => {
                TypeTerm::Seq {
                    seq_repr: if let Some(sr) = seq_repr {
                        Some(Box::new(sr.clone().get_interface_type()))
                    } else { None },
                    item: Box::new(item.get_interface_type())
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

            TypeTerm::Var(varid) => TypeTerm::Var(*varid),
            TypeTerm::Id(tyid) => TypeTerm::Id(*tyid),
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
            TypeTerm::Id(_) => false,
            TypeTerm::Var(_) => false,
            TypeTerm::Num(_) => false,
            TypeTerm::Char(_) => false,
            TypeTerm::Univ{ Γ, bounds, τ } => τ.is_empty(),
            TypeTerm::Spec(ts) |
            TypeTerm::Ladder(ts) |
            TypeTerm::Func(ts) => {
                ts.iter().fold(true, |s,t| s && t.is_empty() )
            }
            TypeTerm::Morph(src,dst) => {
                src.is_empty() && dst.is_empty()
            }
            TypeTerm::Seq{ seq_repr, item } => { item.is_empty() }
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
