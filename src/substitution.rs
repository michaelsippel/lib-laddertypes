
use crate::{
    TypeID,
    TypeTerm
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait Substitution {
    fn get(&self, t: &TypeID) -> Option< TypeTerm >;
}

impl<S: Fn(&TypeID)->Option<TypeTerm>> Substitution for S {
    fn get(&self, t: &TypeID) -> Option< TypeTerm > {
        (self)(t)
    }
}

impl Substitution for std::collections::HashMap< TypeID, TypeTerm > {
    fn get(&self, t: &TypeID) -> Option< TypeTerm > {
        (self as &std::collections::HashMap< TypeID, TypeTerm >).get(t).cloned()
    }
}

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
            TypeTerm::TypeID(typid) => {
                if let Some(t) = σ.get(typid) {
                    *self = t;
                }
            }

            TypeTerm::Ladder(rungs) => {
                for r in rungs.iter_mut() {
                    r.apply_subst(σ);
                }
            }
            TypeTerm::App(args) => {
                for r in args.iter_mut() {
                    r.apply_subst(σ);
                }
            }
            _ => {}
        }

        self
    }
}
