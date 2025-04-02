use {
    crate::{
        substitution_sugared::SugaredSubstitution, sugar::{SugaredStructMember, SugaredTypeTerm}, unification::UnificationProblem, unification_sugared::SugaredUnificationProblem, unparser::*, SugaredEnumVariant, TypeDict, TypeID, TypeTerm
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

    pub fn normalize(&self) -> SugaredMorphismType {
        SugaredMorphismType {
            src_type: self.src_type.clone().normalize(),
            dst_type: self.dst_type.clone().normalize(),
        }
    }

    pub fn strip_common_rungs(&self) -> SugaredMorphismType {
        match (&self.src_type.clone().strip(), &self.dst_type.clone().strip()) {
            (SugaredTypeTerm::Ladder(rungs_lhs), SugaredTypeTerm::Ladder(rungs_rhs)) => {

                let mut lhs_iter = rungs_lhs.iter();
                let mut rhs_iter = rungs_rhs.iter();
                let mut last = SugaredMorphismType { src_type: SugaredTypeTerm::unit(), dst_type: SugaredTypeTerm::unit() };

                while let (Some(lhs_top), Some(rhs_top)) = (lhs_iter.next(), rhs_iter.next()) {
                    last = SugaredMorphismType{
                        src_type: lhs_top.clone(),
                        dst_type: rhs_top.clone()
                    };

                    if lhs_top != rhs_top {
                        let x = SugaredMorphismType{ src_type: lhs_top.clone(), dst_type: rhs_top.clone() }.strip_common_rungs();

                        let mut rl : Vec<_> = lhs_iter.cloned().collect();
                        rl.insert(0, x.src_type);
                        let mut rr : Vec<_> = rhs_iter.cloned().collect();
                        rr.insert(0, x.dst_type);

                        return SugaredMorphismType {
                            src_type: SugaredTypeTerm::Ladder(rl),
                            dst_type: SugaredTypeTerm::Ladder(rr)
                        };
                    }
                }

                last
            }

            (SugaredTypeTerm::Spec(args_lhs), SugaredTypeTerm::Spec(args_rhs)) => {

                let (rl, rr) = args_lhs.iter().zip(args_rhs.iter()).map(
                    |(al,ar)| SugaredMorphismType{ src_type: al.clone(), dst_type: ar.clone() }.strip_common_rungs()
                )
                .fold((vec![], vec![]), |(mut rl, mut rr), x| {
                    rl.push(x.src_type);
                    rr.push(x.dst_type);
                    (rl,rr)
                });

                SugaredMorphismType {
                    src_type: SugaredTypeTerm::Spec(rl),
                    dst_type: SugaredTypeTerm::Spec(rr)
                }
            }

            (SugaredTypeTerm::Seq { seq_repr:seq_repr_lhs, items:items_lhs },
                SugaredTypeTerm::Seq { seq_repr: seq_repr_rhs, items:items_rhs })
            => {
                let (rl, rr) = items_lhs.iter().zip(items_rhs.iter()).map(
                    |(al,ar)| SugaredMorphismType{ src_type: al.clone(), dst_type: ar.clone() }.strip_common_rungs()
                )
                .fold((vec![], vec![]), |(mut rl, mut rr), x| {
                    rl.push(x.src_type);
                    rr.push(x.dst_type);
                    (rl,rr)
                });
                SugaredMorphismType  {
                    src_type: SugaredTypeTerm::Seq{ seq_repr: seq_repr_lhs.clone(), items: rl },
                    dst_type: SugaredTypeTerm::Seq { seq_repr: seq_repr_rhs.clone(), items: rr }
                }
            }

            (SugaredTypeTerm::Struct { struct_repr:struct_repr_lhs, members:members_lhs },
                SugaredTypeTerm::Struct { struct_repr: struct_repr_rhs, members:members_rhs })
            => {
                let mut rl = Vec::new();
                let mut rr = Vec::new();

                for ar in members_rhs.iter() {
                    let mut found = false;
                    for al in members_lhs.iter() {
                        if al.symbol == ar.symbol {
                            let x = SugaredMorphismType{ src_type: al.ty.clone(), dst_type: ar.ty.clone() }.strip_common_rungs();
                            rl.push( SugaredStructMember{
                                symbol: al.symbol.clone(),
                                ty: x.src_type
                            });
                            rr.push( SugaredStructMember{
                                symbol: ar.symbol.clone(),
                                ty: x.dst_type
                            });
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return SugaredMorphismType {
                            src_type: SugaredTypeTerm::Struct { struct_repr: struct_repr_lhs.clone(), members:members_lhs.clone() },
                            dst_type: SugaredTypeTerm::Struct { struct_repr: struct_repr_rhs.clone(), members:members_rhs.clone() }
                        };
                    }
                }

                SugaredMorphismType  {
                    src_type: SugaredTypeTerm::Struct{ struct_repr: struct_repr_lhs.clone(), members: rl },
                    dst_type: SugaredTypeTerm::Struct{ struct_repr: struct_repr_rhs.clone(), members: rr }
                }
            }

            (SugaredTypeTerm::Enum { enum_repr:enum_repr_lhs, variants:variants_lhs },
                SugaredTypeTerm::Enum { enum_repr: enum_repr_rhs, variants:variants_rhs })
            => {
                let mut rl = Vec::new();
                let mut rr = Vec::new();

                for ar in variants_rhs.iter() {
                    let mut found = false;
                    for al in variants_lhs.iter() {
                        if al.symbol == ar.symbol {
                            let x = SugaredMorphismType{ src_type: al.ty.clone(), dst_type: ar.ty.clone() }.strip_common_rungs();
                            rl.push( SugaredEnumVariant{
                                symbol: al.symbol.clone(),
                                ty: x.src_type
                            });
                            rr.push( SugaredEnumVariant{
                                symbol: ar.symbol.clone(),
                                ty: x.dst_type
                            });
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return SugaredMorphismType {
                            src_type: SugaredTypeTerm::Enum { enum_repr: enum_repr_lhs.clone(), variants:variants_lhs.clone() },
                            dst_type: SugaredTypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants:variants_rhs.clone() }
                        };
                    }
                }

                SugaredMorphismType  {
                    src_type: SugaredTypeTerm::Enum{ enum_repr: enum_repr_lhs.clone(), variants: rl },
                    dst_type: SugaredTypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants: rr }
                }
            }

            (x,y) => SugaredMorphismType { src_type: x.clone(), dst_type: y.clone() }
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
    pub fn get_action_type(&self) -> SugaredMorphismType {
        self.get_type().strip_common_rungs()
    }

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
                        src_type: path.first().unwrap().get_type().src_type.clone(),
                        dst_type: path.last().unwrap().get_type().dst_type.clone()
                    }.apply_subst(&s)
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
                    ]),
                    dst_type: SugaredTypeTerm::Ladder(vec![
                        ψ.clone(),
                        SugaredTypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            items: vec![ item_morph.get_type().dst_type ]}
                    ])
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
                        ]),
                    dst_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: dst_struct_repr.clone(),
                                members: member_morph.iter().map(|(symbol, morph)| {
                                    SugaredStructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ])
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
                        ]),
                    dst_type: SugaredTypeTerm::Ladder(vec![ ψ.clone(),
                            SugaredTypeTerm::Struct{
                                struct_repr: enum_repr.clone(),
                                members: variant_morph.iter().map(|(symbol, morph)| {
                                    SugaredStructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                        ])
                }
            }
        }.normalize()
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
