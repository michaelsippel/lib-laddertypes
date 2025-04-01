
use std::ops::DerefMut;
use crate::{
    TypeID,
    TypeTerm
};
use crate::sugar::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait SugaredSubstitution {
    fn get(&self, t: &TypeID) -> Option< SugaredTypeTerm >;
    fn add(&mut self, tyid: TypeID, val: SugaredTypeTerm);
    fn append(self, rhs: &Self) -> Self;
}

impl SugaredSubstitution for std::collections::HashMap< TypeID, SugaredTypeTerm > {
    fn get(&self, t: &TypeID) -> Option< SugaredTypeTerm > {
        (self as &std::collections::HashMap< TypeID, SugaredTypeTerm >).get(t).cloned()
    }

    fn add(&mut self, tyid: TypeID, val: SugaredTypeTerm) {
        if let TypeID::Var(id) = tyid {
            if !val.contains_var(id) {
                self.insert(tyid, val);
            } else {
                eprintln!("substitution cannot contain loop");
            }
        }
    }

    fn append(self, rhs: &Self) -> Self {
        let mut new_σ = std::collections::HashMap::new();
        for (v, tt) in self.iter() {
            let mut tt = tt.clone();
            tt.apply_subst(rhs);
            tt.apply_subst(&self);
            new_σ.add(v.clone(), tt);
        }
        for (v, tt) in rhs.iter() {
            new_σ.add(v.clone(), tt.clone());
        }

        new_σ
    }
}

impl SugaredTypeTerm {
    /// recursively apply substitution to all subterms,
    /// which will replace all occurences of variables which map
    /// some type-term in `subst`
    pub fn apply_substitution(
        &mut self,
        σ: &impl SugaredSubstitution
    ) -> &mut Self {
        self.apply_subst(σ)
    }

    pub fn apply_subst(
        &mut self,
        σ: &impl SugaredSubstitution
    ) -> &mut Self {
        match self {
            SugaredTypeTerm::Num(_) => {},
            SugaredTypeTerm::Char(_) => {},

            SugaredTypeTerm::TypeID(typid) => {
                if let Some(t) = σ.get(typid) {
                    *self = t;
                }
            }
            SugaredTypeTerm::Ladder(args) |
            SugaredTypeTerm::Spec(args) |
            SugaredTypeTerm::Func(args) |
            SugaredTypeTerm::Morph(args)
            => {
                for r in args.iter_mut() {
                    r.apply_subst(σ);
                }
            }

            SugaredTypeTerm::Univ(t) => { t.apply_subst(σ); }

            SugaredTypeTerm::Struct { struct_repr, members } => {
                if let Some(struct_repr) = struct_repr.as_mut() {
                    struct_repr.apply_subst(σ);
                }
                for SugaredStructMember{ symbol:_, ty } in members.iter_mut() {
                    ty.apply_subst(σ);
                }
            },
            SugaredTypeTerm::Enum { enum_repr, variants } => {
                if let Some(enum_repr) = enum_repr.as_mut() {
                    enum_repr.apply_subst(σ);
                }
                for SugaredEnumVariant{ symbol:_, ty } in variants.iter_mut() {
                    ty.apply_subst(σ);
                }
            }
            SugaredTypeTerm::Seq { seq_repr, items } => {
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
