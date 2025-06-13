
use {
    crate::{
        term::{StructMember, TypeTerm},
        context::*,
        morphism::*,
    },
    std::collections::HashMap,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Debug)]
pub enum MorphismInstance<M: Morphism + Clone> {
    Id { τ: TypeTerm },
    Primitive{ m: M },
    Sub {
        ψ: TypeTerm,
        m: Box<MorphismInstance<M>>
    },
    Specialize{
        σ: HashMapSubst,
        m: Box<MorphismInstance<M>>
    },
    Chain{ path: Vec<MorphismInstance<M>> },
    MapSeq{
        seq_repr: Option<Box<TypeTerm>>,
        item_morph: Box<MorphismInstance<M>>,
    },
    MapStruct{
        src_struct_repr: Option<Box<TypeTerm>>,
        dst_struct_repr: Option<Box<TypeTerm>>,
        member_morph: Vec< (String, MorphismInstance<M>) >
    },
    MapEnum{
        enum_repr: Option<Box<TypeTerm>>,
        variant_morph: Vec< (String, MorphismInstance<M>) >
    }
}


impl<M: Morphism + Clone> MorphismInstance<M> {
    pub fn get_action_type(&self) -> MorphismType {
        self.get_type().strip_common_rungs()
    }

    pub fn from_chain(τ: TypeTerm, path: &Vec<MorphismInstance<M>>) -> Self {
        if path.len() == 0 {
            MorphismInstance::Id { τ }
        } else if path.len() == 1 {
            path[0].clone()
        } else {
            MorphismInstance::Chain { path: path.clone() }
        }
    }

    pub fn get_weight(&self) -> u64 {
        match self {
            MorphismInstance::Id { τ } => 0,
            MorphismInstance::Sub { ψ, m } => m.get_weight(),
            MorphismInstance::Specialize { σ, m } => m.get_weight(),
            MorphismInstance::Primitive { m } => 10,
            MorphismInstance::Chain { path } => path.iter().map(|m| m.get_weight()).sum(),
            MorphismInstance::MapSeq {  seq_repr, item_morph } => item_morph.get_weight() + 15,
            MorphismInstance::MapStruct { src_struct_repr, dst_struct_repr, member_morph } => member_morph.iter().map(|m| m.1.get_weight()).sum(),
            MorphismInstance::MapEnum { enum_repr, variant_morph } => variant_morph.iter().map(|m| m.1.get_weight()).sum()
        }
    }

    pub fn get_type(&self) -> MorphismType {
        match self {
            MorphismInstance::Id { τ } => {
                MorphismType {
                    bounds: Vec::new(),
                    src_type: τ.clone(),
                    dst_type: τ.clone()
                }
            }
            MorphismInstance::Primitive { m } => { m.get_type() },
            MorphismInstance::Sub { ψ, m } =>
                MorphismType {
                    bounds: m.get_type().bounds,
                    src_type:
                        TypeTerm::Ladder(vec![
                            ψ.clone(),
                            m.get_type().src_type
                        ]),
                    dst_type: TypeTerm::Ladder(vec![
                            ψ.clone(),
                            m.get_type().dst_type
                        ]),
                },
            MorphismInstance::Specialize { σ, m } =>
                MorphismType {
                    bounds: Vec::new(),
                    src_type: m.get_type().src_type.apply_subst(σ).clone(),
                    dst_type: m.get_type().dst_type.apply_subst(σ).clone(),
                },
            MorphismInstance::Chain { path } => {
                if path.len() > 0 {
                    //let s = self.get_subst();
                    MorphismType {
                        //bounds: path.iter().map(|m| m.get_type().bounds.iter()).flatten().collect(),
                        // here we would need to "move up" the remaining variables
                        bounds: Vec::new(), // <-- fixme: but first implement variable scopes
                        src_type: path.first().unwrap().get_type().src_type.clone(),
                        dst_type: path.last().unwrap().get_type().dst_type.clone()
                    }//.apply_subst(&s)
                } else {
                    MorphismType {
                        bounds: Vec::new(),
                        src_type: TypeTerm::Id(45454),
                        dst_type: TypeTerm::Id(45454)
                    }
                }
            }
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                MorphismType {
                    bounds: item_morph.get_type().bounds,
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().src_type ]},
                    dst_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().dst_type ]},
                }
            }
            MorphismInstance::MapStruct { src_struct_repr, dst_struct_repr, member_morph } => {
                MorphismType {
                    bounds: Vec::new(), // <-- fixme: same as with chain
                    src_type: TypeTerm::Struct{
                                struct_repr: src_struct_repr.clone(),
                                members:
                                    member_morph.iter().map(|(symbol, morph)| {
                                       StructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            },

                    dst_type: TypeTerm::Struct {
                                struct_repr: dst_struct_repr.clone(),
                                members: member_morph.iter().map(|(symbol, morph)| {
                                    StructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                }
            }
            MorphismInstance::MapEnum { enum_repr, variant_morph } => {
                MorphismType {
                    bounds: Vec::new(), // <-- fixme: same as with chain
                    src_type: TypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members:
                                    variant_morph.iter().map(|(symbol, morph)| {
                                       StructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            },
                    dst_type: TypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members: variant_morph.iter().map(|(symbol, morph)| {
                                    StructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                }
            }
        }.normalize()
    }

    pub fn get_subst(&self) -> HashMapSubst {
        match self {
            MorphismInstance::Id { τ } => HashMap::new(),
            MorphismInstance::Primitive { m } => HashMap::new(),
            MorphismInstance::Sub { ψ, m } => m.get_subst(),
            MorphismInstance::Specialize { σ, m } => {
                let mut σ0 = m.get_subst();
                σ0.append(σ);
                σ0
            }
            MorphismInstance::Chain { path } => {
                path.iter().fold(
                    std::collections::HashMap::new(),
                    |mut σ, m| {
                        σ.append(&m.get_subst());
                        σ
                    }
                )
            },
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                item_morph.get_subst()
            },
            MorphismInstance::MapStruct { src_struct_repr, dst_struct_repr, member_morph } => {
                let mut σ = HashMap::new();
                for (symbol, m) in member_morph.iter() {
                    σ.append(&mut m.get_subst());
                }
                σ
            },
            MorphismInstance::MapEnum { enum_repr, variant_morph } => {
                todo!();
                HashMap::new()
            },
        }
    }

    pub fn apply_subst(&mut self, γ: &HashMapSubst) {
        let ty = self.get_type();
        match self {
            MorphismInstance::Id { τ } => {
                τ.apply_subst( γ );
            }
            MorphismInstance::Primitive { m } => { },
            MorphismInstance::Sub { ψ, m } => {
                ψ.apply_subst(γ);
                m.apply_subst(γ);
            }
            MorphismInstance::Specialize { σ, m } => {
                for (n,t) in σ.iter_mut() {
                    t.apply_subst(γ);
                }
                for (i,t) in γ.iter() {
                    if m.get_type().src_type.apply_subst(σ).contains_var(*i)
                    || m.get_type().dst_type.apply_subst(σ).contains_var(*i) {
                        σ.insert(*i, t.clone());
                }

                }
            },
            MorphismInstance::Chain { path } => {
                for n in path.iter_mut() {
                    n.apply_subst(γ);
                }
            }
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                item_morph.apply_subst(γ);
            }
            MorphismInstance::MapStruct { src_struct_repr, dst_struct_repr, member_morph } => {
                for (_,ty) in member_morph {
                    ty.apply_subst(γ);
                }
            },
            MorphismInstance::MapEnum { enum_repr, variant_morph } => {
                for (_,ty) in variant_morph {
                    ty.apply_subst(γ);
                }
            }
        }
    }
}
