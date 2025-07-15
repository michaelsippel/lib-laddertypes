/*
   lib-laddertypes
   Copyright (C) 2023-2025  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use {
    crate::{
        term::{StructMember, TypeTerm},
        context::*,
        morphism::*,
    },
    std::collections::HashMap,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq)]
pub enum MorphismInstance<M: Morphism + Clone> {
    Id { τ: TypeTerm },
    Primitive{ σs: HashMapSubst, m: M },
    Sub {
        ψ: TypeTerm,
        m: Box<MorphismInstance<M>>
    },
    Specialize {
        Γ: ContextPtr,
        m: Box<MorphismInstance<M>>
    },
    Chain {
        path: Vec<MorphismInstance<M>>
    },
    MapSeq {
        seq_repr: Option<Box<TypeTerm>>,
        item_morph: Box<MorphismInstance<M>>,
    },
    MapStruct {
        struct_repr: Option<Box<TypeTerm>>,
        member_morph: Vec< (String, MorphismInstance<M>) >
    },
    MapEnum {
        enum_repr: Option<Box<TypeTerm>>,
        variant_morph: Vec< (String, MorphismInstance<M>) >
    }
}

impl<M: Morphism + Clone> MorphismInstance<M> {

    #[cfg(feature = "pretty")]
    pub fn pretty(&self, Γ: &ContextPtr) -> String {
        let mut s = String::new();

        match self {
            MorphismInstance::Id { τ } => {
                s.push_str( &τ.pretty(&mut Γ.clone(), 0) );
            },
            MorphismInstance::Primitive { σs, m } => {
                let ty = m.get_type().apply_subst(σs);//.apply_subst(Γ);
                s.push_str(&format!("{}\n    -morph->\n{}\n", ty.src_type.pretty(&mut Γ.clone(), 0), ty.dst_type.pretty(&mut Γ.clone(), 0)));
            },
            MorphismInstance::Sub { ψ, m } => {
                s.push_str("Sub {\n");
                s.push_str(&format!("ψ = {}\n", ψ.pretty(&mut Γ.clone(), 0)));
                s.push_str(&m.pretty(Γ));
                s.push_str("}");
            },
            MorphismInstance::Specialize { Γ, m } => {
                s.push_str(&format!("(Γ:{})", Γ.pretty()));
                s.push_str(&m.pretty(Γ));
            },
            MorphismInstance::Chain { path } => {
                s.push_str("Chain {\n");
                for m in path.iter() {
                    s.push_str(&m.pretty(Γ));
                    s.push_str(",");
                }
                s.push_str("}");
            },
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                s.push_str("MapSeq {\n");
                s.push_str(&item_morph.pretty(Γ));
                s.push_str("}");
            },
            MorphismInstance::MapStruct { struct_repr, member_morph } => {
                s.push_str("MapStruct {\n");
                for m in member_morph.iter() {
                    s.push_str(&format!("{} ↦ {}\n", m.0, m.1.pretty(&mut Γ.clone())));
                }
                s.push_str("}");
            },
            MorphismInstance::MapEnum { enum_repr, variant_morph } => {
                s.push_str("MapEnum {\n");
                for m in variant_morph.iter() {
                    s.push_str(&format!("{} ↦ {}\n", m.0, m.1.pretty(&mut Γ.clone())));
                }
                s.push_str("}");
            },
        }

        s
    }

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
            MorphismInstance::Specialize { Γ, m } => m.get_weight(),
            MorphismInstance::Primitive { σs, m } => 10,
            MorphismInstance::Chain { path } => path.iter().map(|m| m.get_weight()).sum(),
            MorphismInstance::MapSeq { seq_repr, item_morph } => item_morph.get_weight() + 15,
            MorphismInstance::MapStruct { struct_repr, member_morph } => member_morph.iter().map(|m| m.1.get_weight()).sum(),
            MorphismInstance::MapEnum { enum_repr, variant_morph } => variant_morph.iter().map(|m| m.1.get_weight()).sum()
        }
    }

    pub fn get_type(&self) -> MorphismType {
        match self {
            MorphismInstance::Id { τ } => {
                MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: τ.clone(),
                    dst_type: τ.clone()
                }
            }
            MorphismInstance::Primitive { σs, m } => { m.get_type().apply_subst(σs) },
            MorphismInstance::Sub { ψ, m } =>
                MorphismType {
                    Γ: Vec::new(),
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
            MorphismInstance::Specialize { Γ, m } => {
                m.get_type().apply_subst(Γ)
            }
            MorphismInstance::Chain { path } => {
                if path.len() > 0 {
                    MorphismType {
                        Γ: Vec::new(),
                        bounds: Vec::new(),
                        src_type: path.first().unwrap().get_type().src_type.clone(),
                        dst_type: path.last().unwrap().get_type().dst_type.clone()
                    }
                } else {
                    unreachable!();
                }
            }
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                MorphismType {
                    Γ: Vec::new(),
                    bounds: item_morph.get_type().bounds,
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            item: Box::new(item_morph.get_type().src_type) },
                    dst_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(),
                            item: Box::new(item_morph.get_type().dst_type) },
                }
            }
            MorphismInstance::MapStruct { struct_repr, member_morph } => {
                MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(), // <-- fixme: same as with chain
                    src_type: TypeTerm::Struct{
                                struct_repr: struct_repr.clone(),
                                members:
                                    member_morph.iter().map(|(symbol, morph)| {
                                       StructMember{ symbol:symbol.clone(), ty: morph.get_type().src_type }
                                    }).collect()
                            },

                    dst_type: TypeTerm::Struct {
                                struct_repr: struct_repr.clone(),
                                members: member_morph.iter().map(|(symbol, morph)| {
                                    StructMember { symbol: symbol.clone(), ty: morph.get_type().dst_type}
                                }).collect()
                            }
                }
            }
            MorphismInstance::MapEnum { enum_repr, variant_morph } => {
                MorphismType {
                    Γ: Vec::new(),
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
            MorphismInstance::Primitive { σs, m } => σs.clone(),
            MorphismInstance::Sub { ψ, m } => m.get_subst(),
            MorphismInstance::Specialize { Γ, m } => {
                todo!();
                HashMap::new()
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
            MorphismInstance::MapStruct { struct_repr, member_morph } => {
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

    pub fn apply_subst(&mut self, γ: &impl Substitution) {
        let ty = self.get_type();
        match self {
            MorphismInstance::Id { τ } => {
                τ.apply_subst( γ );
            }
            MorphismInstance::Primitive { σs, m } => { },
            MorphismInstance::Sub { ψ, m } => {
                ψ.apply_subst(γ);
                m.apply_subst(γ);
            }
            MorphismInstance::Specialize { Γ, m } => {
                todo!();
                /*
                for (v,t) in Γ.0.
                    Γ.bind(*i + Γ.0.read().unwrap().γ.len() as u64, t.clone()).expect("cant bind");
                }
                */
            }
            MorphismInstance::Chain { path } => {
                for n in path.iter_mut() {
                    n.apply_subst(γ);
                }
            }
            MorphismInstance::MapSeq { seq_repr, item_morph } => {
                item_morph.apply_subst(γ);
            }
            MorphismInstance::MapStruct { struct_repr, member_morph } => {
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
