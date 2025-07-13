use crate::term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug)]
pub enum SubstError {
    InvalidVariable,
    UnassignedVariable,
    AlreadyAssigned
}

pub trait Substitution {
    fn saturate(&mut self);
    fn get(&self, t: u64) -> Result<TypeTerm, SubstError>;
}


pub type HashMapSubst = std::collections::HashMap<u64, TypeTerm>;

pub trait SubstitutionMut {
    fn append(&mut self, other: &Self);
    fn filter(self, f: impl FnMut(&(u64, TypeTerm)) -> bool) -> Self;
    fn filter_morphtype(self, ty: &crate::MorphismType) -> Self;
}

impl SubstitutionMut for HashMapSubst {
    fn append(&mut self, other: &HashMapSubst) {
        for (v,t) in other.iter() {
            self.insert(*v,t.clone());
        }
    }

    fn filter(self, f: impl FnMut(&(u64, TypeTerm)) -> bool) -> Self {
        self.into_iter().filter(f).collect()
    }

    fn filter_morphtype(self, ty: &crate::MorphismType) -> Self {
        self.filter(|(v,t)| {
            ty.src_type.contains_var(*v) ||
            ty.dst_type.contains_var(*v)
        })
    }
}

impl Substitution for HashMapSubst {
    fn saturate(&mut self) {
        let mut new_σ = std::collections::HashMap::new();
        for (id, t) in self.iter() {
            let mut t = t.clone();
            t.apply_subst(self);
            new_σ.insert(*id, t.normalize());
        }
        *self = new_σ;
    }

    fn get(&self, t : u64) -> Result<TypeTerm, SubstError> {
        if let Some(t) = (self as &std::collections::HashMap<u64,TypeTerm>).get(&t).cloned() {
            Ok(t)
        } else {
            Err(SubstError::InvalidVariable)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

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
            TypeTerm::Id(_) => {},
            TypeTerm::Num(_) => {},
            TypeTerm::Char(_) => {},

            TypeTerm::Var(var) => {
                if let Ok(t) = σ.get(*var) {
                    *self = t;
                }
            }
            TypeTerm::Ladder(args) |
            TypeTerm::Spec(args) |
            TypeTerm::Func(args)
            => {
                for r in args.iter_mut() {
                    r.apply_subst(σ);
                }
            }

            TypeTerm::Univ(bound, t) => {
                bound.apply_subst(σ);
                t.apply_subst(σ);
            }

            TypeTerm::Morph(src, dst) => {
                src.apply_subst(σ);
                dst.apply_subst(σ);
            }

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
