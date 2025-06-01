use {
    crate::{
        subtype_unify, sugar::SugaredTypeTerm, unification::UnificationProblem, unparser::*, TypeDict, TypeID, TypeTerm
    },
    std::{collections::HashMap, u64}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MorphismType {
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm,
}

impl MorphismType {
    pub fn normalize(self) -> Self {
        MorphismType {
            src_type: self.src_type.normalize().param_normalize(),
            dst_type: self.dst_type.normalize().param_normalize()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait Morphism : Sized {
    fn get_type(&self) -> MorphismType;
    fn map_morphism(&self, seq_type: TypeTerm) -> Option< Self >;

    fn weight(&self) -> u64 {
        1
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq)]
pub struct MorphismInstance<M: Morphism + Clone> {
    pub halo: TypeTerm,
    pub m: M,
    pub σ: HashMap<TypeID, TypeTerm>
}

impl<M: Morphism + Clone> MorphismInstance<M> {
    pub fn get_type(&self) -> MorphismType {
        MorphismType {
            src_type: TypeTerm::Ladder(vec![
                self.halo.clone(),
                self.m.get_type().src_type.clone()
            ]).apply_substitution(&self.σ)
            .clone(),

            dst_type: TypeTerm::Ladder(vec![
                self.halo.clone(),
                self.m.get_type().dst_type.clone()
            ]).apply_substitution(&self.σ)
            .clone()
        }.normalize()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
