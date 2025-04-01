use {
    crate::{
        substitution_sugared::SugaredSubstitution,
        sugar::{SugaredStructMember, SugaredTypeTerm},
        unification::UnificationProblem,
        unification_sugared::SugaredUnificationProblem,
        unparser::*, TypeDict,
        TypeID, TypeTerm
    },
    std::{collections::HashMap, u64}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SugaredMorphismType {
    pub src_type: SugaredTypeTerm,
    pub dst_type: SugaredTypeTerm
}

impl SugaredMorphismType {
    pub fn apply_subst(&self, σ: &impl SugaredSubstitution) -> SugaredMorphismType {
        SugaredMorphismType {
            src_type: self.src_type.clone().apply_subst(σ).clone(),
            dst_type: self.dst_type.clone().apply_subst(σ).clone()
        }
    }
}

pub trait SugaredMorphism : Sized {
    fn get_type(&self) -> SugaredMorphismType;
    fn weight(&self) -> u64 {
        1
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MorphismInstance2<M: SugaredMorphism + Clone> {
    Primitive{
        ψ: SugaredTypeTerm,
        σ: HashMap<TypeID, SugaredTypeTerm>,
        morph: M,
    },
    Chain{
        path: Vec<MorphismInstance2<M>>
    },
    MapSeq{
        ψ: SugaredTypeTerm,
        seq_repr: Option<Box<SugaredTypeTerm>>,
        item_morph: Box<MorphismInstance2<M>>,
    },
    MapStruct{
        ψ: SugaredTypeTerm,
        src_struct_repr: Option<Box<SugaredTypeTerm>>,
        dst_struct_repr: Option<Box<SugaredTypeTerm>>,
        member_morph: Vec< (String, MorphismInstance2<M>) >
    },
    MapEnum{
        ψ: SugaredTypeTerm,
        enum_repr: Option<Box<SugaredTypeTerm>>,
        variant_morph: Vec< (String, MorphismInstance2<M>) >
    }
}


impl<M: SugaredMorphism + Clone> MorphismInstance2<M> {
    pub fn get_type(&self) -> SugaredMorphismType {
        match self {
            MorphismInstance2::Primitive { ψ, σ, morph } => {
                SugaredMorphismType {
                    src_type:
                        SugaredTypeTerm::Ladder(vec![
                            ψ.clone(),
                            morph.get_type().src_type
                                .apply_subst(σ).clone()
                        ]).strip(),

                    dst_type: SugaredTypeTerm::Ladder(vec![
                            ψ.clone(),
                            morph.get_type().dst_type
                                .apply_subst(σ).clone()
                        ]).strip(),
                }
            }
            MorphismInstance2::Chain { path } => {
                if path.len() > 0 {
                    let s = self.get_subst();
                    SugaredMorphismType {
                        src_type: path.first().unwrap().get_type().src_type.clone().apply_subst(&s).clone(),
                        dst_type: path.last().unwrap().get_type().dst_type.clone().apply_subst(&s).clone()
                    }
                } else {
                    SugaredMorphismType {
                        src_type: SugaredTypeTerm::TypeID(TypeID::Fun(45454)),
                        dst_type: SugaredTypeTerm::TypeID(TypeID::Fun(45454))
                    }
                }
            }
            MorphismInstance2::MapSeq { ψ, seq_repr, item_morph } => {
                SugaredMorphismType {
                    src_type: SugaredTypeTerm::Ladder(vec![
                        ψ.clone(),
                        SugaredTypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().src_type ]}
                    ]).strip(),
                    dst_type: SugaredTypeTerm::Ladder(vec![
                        ψ.clone(),
                        SugaredTypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().dst_type ]}
                    ]).strip()
                }
            }
            MorphismInstance2::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                SugaredMorphismType {
                    src_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: src_struct_repr.clone(),
                                members:
                                    member_morph.iter().map(|(symbol, morph)| {
                                       SugaredStructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            }
                        ]).strip(),
                    dst_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: dst_struct_repr.clone(),
                                members: member_morph.iter().map(|(symbol, morph)| {
                                    SugaredStructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ]).strip()
                }
            }
            MorphismInstance2::MapEnum { ψ, enum_repr, variant_morph } => {
                SugaredMorphismType {
                    src_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members:
                                    variant_morph.iter().map(|(symbol, morph)| {
                                       SugaredStructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            }
                        ]).strip(),
                    dst_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members: variant_morph.iter().map(|(symbol, morph)| {
                                    SugaredStructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ]).strip()
                }
            }
        }
    }

    pub fn get_subst(&self) -> std::collections::HashMap< TypeID, SugaredTypeTerm > {
        match self {
            MorphismInstance2::Primitive { ψ, σ, morph } => σ.clone(),
            MorphismInstance2::Chain { path } => {
                path.iter().fold(
                    std::collections::HashMap::new(),
                    |mut σ, m| {
                        σ = σ.append(&m.get_subst());
                        σ
                    }
                )
            },
            MorphismInstance2::MapSeq { ψ, seq_repr, item_morph } => {
                item_morph.get_subst()
            },
            MorphismInstance2::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                let mut σ = HashMap::new();
                for (symbol, m) in member_morph.iter() {
                    σ = σ.append(&mut m.get_subst());
                }
                σ
            },
            MorphismInstance2::MapEnum { ψ, enum_repr, variant_morph } => {
                todo!();
                HashMap::new()
            },
        }
    }

    pub fn apply_subst(&mut self, γ: &std::collections::HashMap< TypeID, SugaredTypeTerm >) {
        let ty = self.get_type();
        match self {
            MorphismInstance2::Primitive { ψ, σ, morph } => {
                ψ.apply_subst(γ);
                for (n,t) in σ.iter_mut() {
                    t.apply_subst(γ);
                }
                for (n,t) in γ.iter() {
                    if let TypeID::Var(varid) = n {
                        if morph.get_type().src_type.apply_subst(σ).contains_var(*varid)
                        || morph.get_type().dst_type.apply_subst(σ).contains_var(*varid) {
                            σ.insert(n.clone(), t.clone());
                        }
                    }
                }
            },
            MorphismInstance2::Chain { path } => {
                for n in path.iter_mut() {
                    n.apply_subst(γ);
                }
            }
            MorphismInstance2::MapSeq { ψ, seq_repr, item_morph } => {
                ψ.apply_subst(γ);
                item_morph.apply_subst(γ);
            }
            MorphismInstance2::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                for (_,ty) in member_morph {
                    ty.apply_subst(γ);
                }
            },
            MorphismInstance2::MapEnum { ψ, enum_repr, variant_morph } => {
                for (_,ty) in variant_morph {
                    ty.apply_subst(γ);
                }
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
