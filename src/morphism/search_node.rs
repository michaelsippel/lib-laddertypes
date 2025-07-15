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
        morphism::DecomposedMorphismType, Context, ContextPtr, EnumVariant, GraphSearch, GraphSearchError, GraphSearchState, HashMapSubst, LayeredContext, Morphism, MorphismBase, MorphismInstance, MorphismType, StructMember, SubstitutionMut, TypeDict, TypeTerm
    },
    std::{collections::HashMap, ops::Deref, sync::{Arc,RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

/// represents a partial path during search in the morphism graph
pub struct SearchNode<M: Morphism+Clone> {
    /// predecessor node
    pub pred: Option< Arc<RwLock< SearchNode<M> >> >,

    /// (measured) weight of the preceding path
    pub weight: u64,

    pub Γ: ContextPtr,
    pub ty: MorphismType,

    /// the advancement over pred
    pub step: Step<M>,
    pub ψ: TypeTerm,
}

pub enum Step<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Inst { m: MorphismInstance<M> },
    MapSeq { seq_repr: Option<Box<TypeTerm>>, item: GraphSearch<M> },
    MapStruct { struct_repr: Option<Box<TypeTerm>>, members: Vec< (String, GraphSearch<M>) > },
    MapEnum { enum_repr: Option<Box<TypeTerm>>, variants: Vec< (String, GraphSearch<M>) > }
}

#[derive(Debug)]
pub enum SolvedStep<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Inst { m: MorphismInstance<M> },
    MapSeq { seq_repr: Option<Box<TypeTerm>>, item: MorphismInstance<M> },
    MapStruct { struct_repr: Option<Box<TypeTerm>>, members: Vec< (String, MorphismInstance<M>) > },
    MapEnum { enum_repr: Option<Box<TypeTerm>>, variants: Vec< (String, MorphismInstance<M>) > }
}


//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait SearchNodeExt<M: Morphism+Clone> {
   // fn specialize(&self, σ: HashMapSubst) -> Arc<RwLock<SearchNode<M>>>;
    fn chain(&self, ψ: TypeTerm, Γ: &ContextPtr, σs: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>>;
    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>>;
    fn map_seq(&self, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>>;
    fn map_struct(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;
    fn map_enum(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;

    fn advance(&self, base: &MorphismBase<M>) -> Result<bool, GraphSearchError>;
    fn to_morphism_instance(&self) -> Option< MorphismInstance<M> >;

    fn is_ready(&self) -> bool;
    fn get_weight(&self) -> u64;
    fn get_type(&self) -> MorphismType;

    fn creates_loop(&self) -> bool;
}

impl<M: Morphism+Clone> SearchNodeExt<M> for Arc<RwLock<SearchNode<M>>> {
    fn get_weight(&self) -> u64 {
        self.read().unwrap().weight
        + match &self.read().unwrap().step {
            Step::Id { τ } => 0,
            Step::Inst { m } => 1+m.get_weight(),
            Step::MapSeq { seq_repr, item } => item.best_path_weight(),
            Step::MapStruct { struct_repr, members } => members.iter().map(|(_,g)| g.best_path_weight() ).sum(),
            Step::MapEnum { enum_repr, variants } => variants.iter().map(|(_,g)| g.best_path_weight() ).max().unwrap_or(0),
        }
    }

    fn get_type(&self) -> MorphismType {
        let s = self.read().unwrap();
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.src_type.clone() ]).normalize(),
            dst_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.dst_type.clone() ]).normalize(),
        }.apply_subst(&s.Γ)
    }

    // tell if this sub-search already has a solution
    fn is_ready(&self) -> bool {
        let n = self.read().unwrap();
        match &n.step {
            Step::Id { τ } => true,
            Step::MapSeq { seq_repr, item } => {
                item.get_solution().is_some()
            }
            Step::MapStruct { struct_repr, members } => {
                members.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
            Step::MapEnum { enum_repr, variants } => {
                variants.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
            Step::Inst { m } => true
        }
    }

    fn creates_loop(&self) -> bool {
        let mut cur_node = self.read().unwrap().pred.clone();
        while let Some(n) = cur_node {
            let s = &n.read().unwrap().step;
            match s {
                Step::Id { τ } => {}
                _ => {
                    //              V- dst_type ?
                    if n.get_type().src_type == self.get_type().dst_type {
                        return true;
                    }
                }
            }

            cur_node = n.read().unwrap().pred.clone();
        }

        false
    }

    fn advance(&self, base: &MorphismBase<M>) -> Result<bool, GraphSearchError> {
        let mut n = self.write().unwrap();
        match &mut n.step {
            Step::MapSeq { seq_repr, item } => {
                //eprintln!("advance seq-map");
                match item.advance(base) {
                    GraphSearchState::Solved(item_morph) => {
                        //eprintln!("Sequence-Map Sub Graph Solved!!");
                        n.ty = MorphismType {
                            Γ: Vec::new(),
                            bounds: Vec::new(),
                            src_type: TypeTerm::Seq { seq_repr: seq_repr.clone(), item: Box::new(item_morph.get_type().src_type) },
                            dst_type: TypeTerm::Seq { seq_repr: seq_repr.clone(), item: Box::new(item_morph.get_type().dst_type) },
                        };
                        Ok(false)
                    }
                    GraphSearchState::Continue => Ok(true),
                    GraphSearchState::Err(err) => Err(err)
                }
            }
            Step::MapStruct { struct_repr, members } => {
                for (symbol, sub_search) in members.iter_mut() {
                    if sub_search.get_solution().is_none() {
                        match sub_search.advance(base) {
                            GraphSearchState::Solved(_) => {
                                return Ok(true);
                            },
                            GraphSearchState::Continue => { return Ok(true); },
                            GraphSearchState::Err(err) => { return Err(err); }
                        }
                    } else {
                        // already solved, continue with next member
                    }
                }

                // all sub searches are solved
                n.ty = MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: members.iter().map(|(s,g)| StructMember{ symbol:s.clone(), ty: g.get_solution().unwrap().get_type().src_type }).collect() },
                    dst_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: members.iter().map(|(s,g)| StructMember{ symbol:s.clone(), ty: g.get_solution().unwrap().get_type().dst_type }).collect() },
                };
                return Ok(false);
            }
            Step::MapEnum { enum_repr, variants } => {
                for (symbol, sub_search) in variants.iter_mut() {
                    if sub_search.get_solution().is_none() {
                        match sub_search.advance(base) {
                            GraphSearchState::Solved(_) => {
                                return Ok(true);
                            },
                            GraphSearchState::Continue => { return Ok(true); },
                            GraphSearchState::Err(err) => { return Err(err); }
                        }
                    }
                }

                // all sub searches are solved
                n.ty = MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: variants.iter().map(|(s,g)| EnumVariant{ symbol:s.clone(), ty: g.get_solution().unwrap().get_type().src_type }).collect() },
                    dst_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: variants.iter().map(|(s,g)| EnumVariant{ symbol:s.clone(), ty: g.get_solution().unwrap().get_type().dst_type }).collect() },
                };
                return Ok(false);
            }
            _ => Ok(false)
        }
    }

    fn chain(&self, ψ: TypeTerm, Γinst: &ContextPtr, σs: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>> {
        let m = MorphismInstance::Primitive { σs: σs.clone(), m: m.clone() };

        let mut src_type = self.get_type().src_type;
        src_type.apply_subst(&Γinst.shift_from_parent());

        let dst_type = m.get_type().dst_type;

        let n = Arc::new(RwLock::new(SearchNode {
            Γ: Γinst.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType { Γ: Vec::new(), bounds: Vec::new(), src_type, dst_type },
            step: Step::Inst{ m },
            ψ: TypeTerm::unit(),
        }));
        n.set_sub(ψ);

        n
    }

    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>> {
        let oldψ = &mut self.write().unwrap().ψ;
        *oldψ = TypeTerm::Ladder(vec![ ψ, oldψ.clone() ]).normalize();
        self.clone()
    }

    fn map_seq(&self, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>> {

        let seq_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Seq { seq_repr, item } => {
                seq_repr.clone()
            }
            _ => unreachable!()
        };

        Arc::new(RwLock::new(SearchNode {
            Γ: self.read().unwrap().Γ.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(), item: Box::new(goal.src_type.clone()) },
                    dst_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(), item: Box::new(goal.src_type.clone()) }
                },
            step: Step::MapSeq { seq_repr, item: GraphSearch::new(self.read().unwrap().Γ.scope(), goal) },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_struct(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {

        let struct_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Struct { struct_repr, members } => {
                struct_repr.clone()
            }
            _ => unreachable!()
        };

        Arc::new(RwLock::new(SearchNode {
            Γ: self.read().unwrap().Γ.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                Γ: Vec::new(),
                bounds:Vec::new(),
                src_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapStruct {
                struct_repr,
                members: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(self.read().unwrap().Γ.scope(), goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_enum(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {
        let enum_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Enum { enum_repr, variants } => {
                enum_repr.clone()
            }
            _ => unreachable!()
        };

        Arc::new(RwLock::new(SearchNode {
            Γ: self.read().unwrap().Γ.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                Γ: Vec::new(),
                bounds: Vec::new(),
                src_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapEnum { enum_repr, variants: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(self.read().unwrap().Γ.scope(), goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }


    fn to_morphism_instance(&self) -> Option< MorphismInstance<M> > {
        let mut steps = Vec::new();
        let mut cur_node = Some(self.clone());

        let mut Γ = self.read().unwrap().Γ.clone();

        let mut offset = 0;
        while let Some(n) = cur_node {
            let n = n.read().unwrap();
            steps.push((n.ψ.clone(),
                match &n.step {
                    Step::Id { τ } => SolvedStep::Id { τ: τ.clone() },
                    Step::Inst { m } => SolvedStep::Inst { m: m.clone() },
                    Step::MapSeq { seq_repr, item } => {
                        let mut item = item.get_solution().unwrap();
                        //item.apply_subst(&n.Γ.0.read().unwrap().σ);
                        SolvedStep::MapSeq {
                            seq_repr: seq_repr.clone(),
                            item,
                        }
                    },
                    Step::MapStruct { struct_repr, members } => SolvedStep::MapStruct {
                        struct_repr: struct_repr.clone(),
                        members: members.iter().map(|(n,m)| (n.clone(), m.get_solution().unwrap())).collect()
                    },
                    Step::MapEnum { enum_repr, variants } => SolvedStep::MapEnum {
                        enum_repr: enum_repr.clone(),
                        variants: variants.iter().map(|(n,m)| (n.clone(), m.get_solution().unwrap())).collect()
                    },
                }));

            //let σs = Γ.shift_variables(&n.Γ);
            //let sigma = n.Γ.0.read().unwrap().σ.clone();
            //offset += n.Γ.0.read().unwrap().γ.len() as u64;
            /*
            for (v,t) in sigma.into_iter() {
                Γ.bind(v, t).expect("cant bind");
            }
            */

            cur_node = n.pred.clone();
        }

        steps.reverse();

        let mut begin = TypeTerm::unit();
        let mut path = Vec::new();
        //eprintln!("to_morph_instance:\n==");
        for (ψ, s) in steps {
            match s {
                SolvedStep::Id { τ } => {
                    //eprintln!("to_morph_instance: ID {:?}", τ);
                    begin = τ.clone();
                }
                SolvedStep::Inst{ m } => {
                    eprintln!("to_morph_instance: Inst {:?} -- {:?}", ψ, m.get_type());
                    let mut m = m.clone();

                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m.clone());
                }
                SolvedStep::MapSeq { seq_repr, item } => {
                    let mut m = MorphismInstance::MapSeq {
                        seq_repr: seq_repr.clone(),
                        item_morph: Box::new(item)
                    };

                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
                SolvedStep::MapStruct { struct_repr, members } => {
                    let mut m = MorphismInstance::MapStruct {
                        struct_repr: struct_repr.clone(),
                        member_morph: members
                    };
                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
                SolvedStep::MapEnum { enum_repr, variants } => {
                    let mut m = MorphismInstance::MapEnum {
                        enum_repr: enum_repr.clone(),
                        variant_morph: variants
                    };
                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
            }
        }

        eprintln!("to_morphism_instance: Γ: {}", Γ.pretty());

        if Γ.0.read().unwrap().n_variables() > 0 {
            Some(
                MorphismInstance::Specialize {
                    Γ: Γ.clone(),
                    m: Box::new(MorphismInstance::from_chain(begin, &path))
                }
            )
        } else {
            Some(MorphismInstance::from_chain(begin, &path))
        }
    }
}
