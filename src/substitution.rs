
use std::ops::DerefMut;
use crate::{
    TypeID,
    DesugaredTypeTerm
};
use crate::term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait Substitution {
    fn get(&self, t: &TypeID) -> Option< TypeTerm >;
    fn add(&mut self, tyid: TypeID, val: TypeTerm);
    fn append(self, rhs: &Self) -> Self;
}

impl Substitution for std::collections::HashMap< TypeID, TypeTerm > {
    fn get(&self, t: &TypeID) -> Option< TypeTerm > {
        (self as &std::collections::HashMap< TypeID, TypeTerm >).get(t).cloned()
    }

    fn add(&mut self, tyid: TypeID, val: TypeTerm) {
        if let TypeID::Var(id) = tyid {
            if !val.contains_var(id) {
                self.insert(tyid, val.normalize());
            } else {
                eprintln!("substitution cannot contain loop");
            }
        }
    }

    fn append(self, rhs: &Self) -> Self {
        let mut new_σ = std::collections::HashMap::new();
        for (v, tt) in self.iter() {
            let mut tt = tt.clone().normalize();
            tt.apply_subst(rhs);
            tt.apply_subst(&self);
            new_σ.add(v.clone(), tt);
        }
        for (v, tt) in rhs.iter() {
            new_σ.add(v.clone(), tt.clone().normalize());
        }

        new_σ
    }
}

pub type HashMapSubst = std::collections::HashMap< TypeID, TypeTerm >;

impl TypeTerm {
    /// recursively apply substitution to all subterms,
    /// which will replace all occurences of variables which map
    /// some type-term in `subst`
    pub fn apply_substitution(
        &mut self,
        σ: &impl Substitution
    ) -> &mut Self {
        self.apply_subst(σ)
    }

    pub fn apply_subst(
        &mut self,
        σ: &impl Substitution
    ) -> &mut Self {
        match self {
            TypeTerm::Num(_) => {},
            TypeTerm::Char(_) => {},

            TypeTerm::TypeID(typid) => {
                if let Some(t) = σ.get(typid) {
                    *self = t;
                }
            }
            TypeTerm::Ladder(args) |
            TypeTerm::Spec(args) |
            TypeTerm::Func(args) |
            TypeTerm::Morph(args)
            => {
                for r in args.iter_mut() {
                    r.apply_subst(σ);
                }
            }

            TypeTerm::Univ(t) => { t.apply_subst(σ); }

            TypeTerm::Struct { struct_repr, members } => {
                if let Some(struct_repr) = struct_repr.as_mut() {
                    struct_repr.apply_subst(σ);
                }
                for StructMember{ symbol:_, ty } in members.iter_mut() {
                    ty.apply_subst(σ);
                }
            },
            TypeTerm::Enum { enum_repr, variants } => {
                if let Some(enum_repr) = enum_repr.as_mut() {
                    enum_repr.apply_subst(σ);
                }
                for EnumVariant{ symbol:_, ty } in variants.iter_mut() {
                    ty.apply_subst(σ);
                }
            }
            TypeTerm::Seq { seq_repr, items } => {
                if let Some(seq_repr) = seq_repr {
                    seq_repr.apply_subst(σ);
                }
                for ty in items.iter_mut() {
                    ty.apply_subst(σ);
                }
            },
        }

        self
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
