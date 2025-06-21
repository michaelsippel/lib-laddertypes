pub mod base;
pub mod search_node;
pub mod graph;
pub mod instance;
pub mod heuristic;

use std::ops::Deref;

pub use base::*;
pub use graph::*;
pub use instance::*;

use crate::{ConstraintPair, CP2};

use {
    crate::{
        constraint_system::ConstraintSystem, substitution::Substitution, term::{StructMember, TypeTerm},
        unparser::*, EnumVariant, TypeDict, TypeID, VariableConstraint,
        context::*
    },
    std::{
        collections::HashMap,
        sync::{Arc, RwLock}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait Morphism : Sized {
    fn ctx(&self) -> ContextPtr;
    fn get_type(&self) -> MorphismType;
    fn weight(&self) -> u64 {
        1
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MorphismType {
    pub Γ: Vec< ContextEntry >,
    pub bounds: Vec< ConstraintPair >,
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm
}

impl MorphismType {
    pub fn strip_common_rungs(&self) -> MorphismType {
        match (&self.src_type.clone().strip(), &self.dst_type.clone().strip()) {
            (TypeTerm::Ladder(rungs_lhs), TypeTerm::Ladder(rungs_rhs)) => {

                let mut lhs_iter = rungs_lhs.iter();
                let mut rhs_iter = rungs_rhs.iter();
                let mut last = MorphismType {
                    Γ: self.Γ.clone(),
                    bounds: self.bounds.clone(),
                    src_type: TypeTerm::unit(),
                    dst_type: TypeTerm::unit()
                };

                while let (Some(lhs_top), Some(rhs_top)) = (lhs_iter.next(), rhs_iter.next()) {
                    last.src_type = lhs_top.clone();
                    last.dst_type = rhs_top.clone();

                    if lhs_top != rhs_top {
                        let x = MorphismType {
                            Γ: self.Γ.clone(),
                            bounds: self.bounds.clone(),
                            src_type: lhs_top.clone(),
                            dst_type: rhs_top.clone()
                        }.strip_common_rungs();

                        let mut rl : Vec<_> = lhs_iter.cloned().collect();
                        rl.insert(0, x.src_type);
                        let mut rr : Vec<_> = rhs_iter.cloned().collect();
                        rr.insert(0, x.dst_type);

                        return MorphismType {
                            Γ: self.Γ.clone(),
                            bounds: self.bounds.clone(),
                            src_type: TypeTerm::Ladder(rl),
                            dst_type: TypeTerm::Ladder(rr)
                        };
                    }
                }

                last
            }

            (TypeTerm::Spec(args_lhs), TypeTerm::Spec(args_rhs)) => {

                let (rl, rr) = args_lhs.iter().zip(args_rhs.iter()).map(
                    |(al,ar)| MorphismType{
                        Γ: self.Γ.clone(),
                        bounds: self.bounds.clone(),
                        src_type: al.clone(),
                        dst_type: ar.clone()
                    }.strip_common_rungs()
                )
                .fold((vec![], vec![]), |(mut rl, mut rr), x| {
                    rl.push(x.src_type);
                    rr.push(x.dst_type);
                    (rl,rr)
                });

                MorphismType {
                    Γ: self.Γ.clone(),
                    bounds: self.bounds.clone(),
                    src_type: TypeTerm::Spec(rl),
                    dst_type: TypeTerm::Spec(rr)
                }
            }

            (TypeTerm::Seq { seq_repr:seq_repr_lhs, item:item_lhs },
                TypeTerm::Seq { seq_repr: seq_repr_rhs, item:item_rhs })
            => {
                let i = MorphismType{
                        Γ: self.Γ.clone(),
                        bounds: self.bounds.clone(),
                        src_type: item_lhs.deref().clone(),
                        dst_type: item_rhs.deref().clone()
                    }.strip_common_rungs();

                MorphismType  {
                    Γ: self.Γ.clone(),
                    bounds: self.bounds.clone(),
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr_lhs.clone(), item: Box::new(i.src_type) },
                    dst_type: TypeTerm::Seq { seq_repr: seq_repr_rhs.clone(), item: Box::new(i.dst_type) }
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
                            let x = MorphismType{
                                Γ: self.Γ.clone(),
                                bounds: self.bounds.clone(),
                                src_type: al.ty.clone(),
                                dst_type: ar.ty.clone()
                            }.strip_common_rungs();

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
                            Γ: self.Γ.clone(),
                            bounds: self.bounds.clone(),
                            src_type: TypeTerm::Struct { struct_repr: struct_repr_lhs.clone(), members:members_lhs.clone() },
                            dst_type: TypeTerm::Struct { struct_repr: struct_repr_rhs.clone(), members:members_rhs.clone() }
                        };
                    }
                }

                MorphismType  {
                    Γ: self.Γ.clone(),
                    bounds: self.bounds.clone(),
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
                            let x = MorphismType {
                                Γ: self.Γ.clone(),
                                bounds: self.bounds.clone(),
                                src_type: al.ty.clone(),
                                dst_type: ar.ty.clone()
                            }.strip_common_rungs();

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
                            Γ: self.Γ.clone(),
                            bounds: self.bounds.clone(),
                            src_type: TypeTerm::Enum { enum_repr: enum_repr_lhs.clone(), variants:variants_lhs.clone() },
                            dst_type: TypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants:variants_rhs.clone() }
                        };
                    }
                }

                MorphismType  {
                    Γ: self.Γ.clone(),
                    bounds: self.bounds.clone(),
                    src_type: TypeTerm::Enum{ enum_repr: enum_repr_lhs.clone(), variants: rl },
                    dst_type: TypeTerm::Enum { enum_repr: enum_repr_rhs.clone(), variants: rr }
                }
            }

            (x,y) => MorphismType {
                Γ: self.Γ.clone(),
                bounds: self.bounds.clone(),
                src_type: x.clone(),
                dst_type: y.clone()
            }
        }
    }

    pub fn apply_subst(&self, σ: &impl Substitution) -> MorphismType {
        MorphismType {
            Γ: self.Γ.clone(),
            bounds: self.bounds.iter().map(|cp| cp.clone().apply_subst(σ).clone()).collect(),
            src_type: self.src_type.clone().apply_subst(σ).clone(),
            dst_type: self.dst_type.clone().apply_subst(σ).clone()
        }
    }

    pub fn normalize(&self) -> MorphismType {
        MorphismType {
            Γ: self.Γ.clone(),
            bounds: self.bounds.iter().map(|cp| cp.normalize()).collect(),
            src_type: self.src_type.clone().normalize(),
            dst_type: self.dst_type.clone().normalize(),
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
