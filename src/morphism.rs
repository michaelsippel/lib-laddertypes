use {
    crate::{
        substitution::Substitution, term::{StructMember, TypeTerm},
        constraint_system::ConstraintSystem, unparser::*, EnumVariant, TypeDict, TypeID
    },
    std::{collections::HashMap, u64}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MorphismType {
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm
}

impl MorphismType {
    pub fn strip_common_rungs(&self) -> MorphismType {
        match (&self.src_type.clone().strip(), &self.dst_type.clone().strip()) {
            (TypeTerm::Ladder(rungs_lhs), TypeTerm::Ladder(rungs_rhs)) => {

                let mut lhs_iter = rungs_lhs.iter();
                let mut rhs_iter = rungs_rhs.iter();
                let mut last = MorphismType { src_type: TypeTerm::unit(), dst_type: TypeTerm::unit() };

                while let (Some(lhs_top), Some(rhs_top)) = (lhs_iter.next(), rhs_iter.next()) {
                    last = MorphismType{
                        src_type: lhs_top.clone(),
                        dst_type: rhs_top.clone()
                    };

                    if lhs_top != rhs_top {
                        let x = MorphismType{ src_type: lhs_top.clone(), dst_type: rhs_top.clone() }.strip_common_rungs();

                        let mut rl : Vec<_> = lhs_iter.cloned().collect();
                        rl.insert(0, x.src_type);
                        let mut rr : Vec<_> = rhs_iter.cloned().collect();
                        rr.insert(0, x.dst_type);

                        return MorphismType {
                            src_type: TypeTerm::Ladder(rl),
                            dst_type: TypeTerm::Ladder(rr)
                        };
                    }
                }

                last
            }

            (TypeTerm::Spec(args_lhs), TypeTerm::Spec(args_rhs)) => {

                let (rl, rr) = args_lhs.iter().zip(args_rhs.iter()).map(
                    |(al,ar)| MorphismType{ src_type: al.clone(), dst_type: ar.clone() }.strip_common_rungs()
                )
                .fold((vec![], vec![]), |(mut rl, mut rr), x| {
                    rl.push(x.src_type);
                    rr.push(x.dst_type);
                    (rl,rr)
                });

                MorphismType {
                    src_type: TypeTerm::Spec(rl),
                    dst_type: TypeTerm::Spec(rr)
                }
            }

            (TypeTerm::Seq { seq_repr:seq_repr_lhs, items:items_lhs },
                TypeTerm::Seq { seq_repr: seq_repr_rhs, items:items_rhs })
            => {
                let (rl, rr) = items_lhs.iter().zip(items_rhs.iter()).map(
                    |(al,ar)| MorphismType{ src_type: al.clone(), dst_type: ar.clone() }.strip_common_rungs()
                )
                .fold((vec![], vec![]), |(mut rl, mut rr), x| {
                    rl.push(x.src_type);
                    rr.push(x.dst_type);
                    (rl,rr)
                });
                MorphismType  {
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr_lhs.clone(), items: rl },
                    dst_type: TypeTerm::Seq { seq_repr: seq_repr_rhs.clone(), items: rr }
                }
            }

            (TypeTerm::Struct { struct_repr:struct_repr_lhs, members:members_lhs },
                TypeTerm::Struct { struct_repr: struct_repr_rhs, members:members_rhs })
            => {
                let mut rl = Vec::new();
                let mut rr = Vec::new();

                for ar in members_rhs.iter() {
                    let mut found = false;
                    for al in members_lhs.iter() {
                        if al.symbol == ar.symbol {
                            let x = MorphismType{ src_type: al.ty.clone(), dst_type: ar.ty.clone() }.strip_common_rungs();
                            rl.push( StructMember{
                                symbol: al.symbol.clone(),
                                ty: x.src_type
                            });
                            rr.push( StructMember{
                                symbol: ar.symbol.clone(),
                                ty: x.dst_type
                            });
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return MorphismType {
                            src_type: TypeTerm::Struct { struct_repr: struct_repr_lhs.clone(), members:members_lhs.clone() },
                            dst_type: TypeTerm::Struct { struct_repr: struct_repr_rhs.clone(), members:members_rhs.clone() }
                        };
                    }
                }

                MorphismType  {
                    src_type: TypeTerm::Struct{ struct_repr: struct_repr_lhs.clone(), members: rl },
                    dst_type: TypeTerm::Struct{ struct_repr: struct_repr_rhs.clone(), members: rr }
                }
            }

            (TypeTerm::Enum { enum_repr:enum_repr_lhs, variants:variants_lhs },
                TypeTerm::Enum { enum_repr: enum_repr_rhs, variants:variants_rhs })
            => {
                let mut rl = Vec::new();
                let mut rr = Vec::new();

                for ar in variants_rhs.iter() {
                    let mut found = false;
                    for al in variants_lhs.iter() {
                        if al.symbol == ar.symbol {
                            let x = MorphismType{ src_type: al.ty.clone(), dst_type: ar.ty.clone() }.strip_common_rungs();
                            rl.push( EnumVariant{
                                symbol: al.symbol.clone(),
                                ty: x.src_type
                            });
                            rr.push( EnumVariant{
                                symbol: ar.symbol.clone(),
                                ty: x.dst_type
                            });
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return MorphismType {
                            src_type: TypeTerm::Enum { enum_repr: enum_repr_lhs.clone(), variants:variants_lhs.clone() },
                            dst_type: TypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants:variants_rhs.clone() }
                        };
                    }
                }

                MorphismType  {
                    src_type: TypeTerm::Enum{ enum_repr: enum_repr_lhs.clone(), variants: rl },
                    dst_type: TypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants: rr }
                }
            }

            (x,y) => MorphismType { src_type: x.clone(), dst_type: y.clone() }
        }
    }

    pub fn apply_subst(&self, σ: &impl Substitution) -> MorphismType {
        MorphismType {
            src_type: self.src_type.clone().apply_subst(σ).clone(),
            dst_type: self.dst_type.clone().apply_subst(σ).clone()
        }
    }


    pub fn normalize(&self) -> MorphismType {
        MorphismType {
            src_type: self.src_type.clone().normalize(),
            dst_type: self.dst_type.clone().normalize(),
        }
    }
}

pub trait Morphism : Sized {
    fn get_type(&self) -> MorphismType;
    fn weight(&self) -> u64 {
        1
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum MorphismInstance<M: Morphism + Clone> {
    Primitive{
        ψ: TypeTerm,
        σ: HashMap<TypeID, TypeTerm>,
        morph: M,
    },
    Chain{
        path: Vec<MorphismInstance<M>>
    },
    MapSeq{
        ψ: TypeTerm,
        seq_repr: Option<Box<TypeTerm>>,
        item_morph: Box<MorphismInstance<M>>,
    },
    MapStruct{
        ψ: TypeTerm,
        src_struct_repr: Option<Box<TypeTerm>>,
        dst_struct_repr: Option<Box<TypeTerm>>,
        member_morph: Vec< (String, MorphismInstance<M>) >
    },
    MapEnum{
        ψ: TypeTerm,
        enum_repr: Option<Box<TypeTerm>>,
        variant_morph: Vec< (String, MorphismInstance<M>) >
    }
}


impl<M: Morphism + Clone> MorphismInstance<M> {
    pub fn get_action_type(&self) -> MorphismType {
        self.get_type().strip_common_rungs()
    }

    pub fn get_type(&self) -> MorphismType {
        match self {
            MorphismInstance::Primitive { ψ, σ, morph } => {
                MorphismType {
                    src_type:
                        TypeTerm::Ladder(vec![
                            ψ.clone(),
                            morph.get_type().src_type
                                .apply_subst(σ).clone()
                        ]).strip(),

                    dst_type: TypeTerm::Ladder(vec![
                            ψ.clone(),
                            morph.get_type().dst_type
                                .apply_subst(σ).clone()
                        ]).strip(),
                }
            }
            MorphismInstance::Chain { path } => {
                if path.len() > 0 {
                    let s = self.get_subst();
                    MorphismType {
                        src_type: path.first().unwrap().get_type().src_type.clone(),
                        dst_type: path.last().unwrap().get_type().dst_type.clone()
                    }.apply_subst(&s)
                } else {
                    MorphismType {
                        src_type: TypeTerm::TypeID(TypeID::Fun(45454)),
                        dst_type: TypeTerm::TypeID(TypeID::Fun(45454))
                    }
                }
            }
            MorphismInstance::MapSeq { ψ, seq_repr, item_morph } => {
                MorphismType {
                    src_type: TypeTerm::Ladder(vec![
                        ψ.clone(),
                        TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().src_type ]}
                    ]),
                    dst_type: TypeTerm::Ladder(vec![
                        ψ.clone(),
                        TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().dst_type ]}
                    ])
                }
            }
            MorphismInstance::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                MorphismType {
                    src_type: TypeTerm::Ladder(vec![ ψ.clone(),
                            TypeTerm::Struct{
                                struct_repr: src_struct_repr.clone(),
                                members:
                                    member_morph.iter().map(|(symbol, morph)| {
                                       StructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            }
                        ]),
                    dst_type: TypeTerm::Ladder(vec![ ψ.clone(),
                            TypeTerm::Struct{
                                struct_repr: dst_struct_repr.clone(),
                                members: member_morph.iter().map(|(symbol, morph)| {
                                    StructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ])
                }
            }
            MorphismInstance::MapEnum { ψ, enum_repr, variant_morph } => {
                MorphismType {
                    src_type: TypeTerm::Ladder(vec![ ψ.clone(),
                            TypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members:
                                    variant_morph.iter().map(|(symbol, morph)| {
                                       StructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            }
                        ]),
                    dst_type: TypeTerm::Ladder(vec![ ψ.clone(),
                            TypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members: variant_morph.iter().map(|(symbol, morph)| {
                                    StructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ])
                }
            }
        }.normalize()
    }

    pub fn get_subst(&self) -> std::collections::HashMap< TypeID, TypeTerm > {
        match self {
            MorphismInstance::Primitive { ψ, σ, morph } => σ.clone(),
            MorphismInstance::Chain { path } => {
                path.iter().fold(
                    std::collections::HashMap::new(),
                    |mut σ, m| {
                        σ = σ.append(&m.get_subst());
                        σ
                    }
                )
            },
            MorphismInstance::MapSeq { ψ, seq_repr, item_morph } => {
                item_morph.get_subst()
            },
            MorphismInstance::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                let mut σ = HashMap::new();
                for (symbol, m) in member_morph.iter() {
                    σ = σ.append(&mut m.get_subst());
                }
                σ
            },
            MorphismInstance::MapEnum { ψ, enum_repr, variant_morph } => {
                todo!();
                HashMap::new()
            },
        }
    }

    pub fn apply_subst(&mut self, γ: &std::collections::HashMap< TypeID, TypeTerm >) {
        let ty = self.get_type();
        match self {
            MorphismInstance::Primitive { ψ, σ, morph } => {
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
            MorphismInstance::Chain { path } => {
                for n in path.iter_mut() {
                    n.apply_subst(γ);
                }
            }
            MorphismInstance::MapSeq { ψ, seq_repr, item_morph } => {
                ψ.apply_subst(γ);
                item_morph.apply_subst(γ);
            }
            MorphismInstance::MapStruct { ψ, src_struct_repr, dst_struct_repr, member_morph } => {
                for (_,ty) in member_morph {
                    ty.apply_subst(γ);
                }
            },
            MorphismInstance::MapEnum { ψ, enum_repr, variant_morph } => {
                for (_,ty) in variant_morph {
                    ty.apply_subst(γ);
                }
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
