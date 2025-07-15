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
        morphism::DecomposedMorphismType, subtype_unify, unify, AddressingMode, Context, ContextPtr, EnumVariant, GraphSearch, GraphSearchError, GraphSearchState, HashMapSubst, LayeredContext, Morphism, MorphismBase, MorphismInstance, MorphismType, StructMember, Substitution, SubstitutionMut, TypeDict, TypeTerm
    },
    std::{cmp::Ordering, sync::{Arc,RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

/// represents a partial path during search in the morphism graph
pub struct SearchNode<M: Morphism+Clone> {
    pub id: u64,

    /// predecessor node
    pub pred: Option< Arc<RwLock< SearchNode<M> >> >,

    /// (measured) weight of the preceding path
    pub weight: u64,

    /// (estimated) remaining weight to complete this path
    pub est_remain: u64,

    pub ctx: ContextPtr,
    pub ty: MorphismType,

    /// the advancement over pred
    pub step: Step<M>,
    pub ψ: TypeTerm,
}

pub enum Step<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Prim { σs: HashMapSubst, m: M },
    MapSeq { seq_repr: Option<Box<TypeTerm>>, item: GraphSearch<M> },
    MapStruct { struct_repr: Option<Box<TypeTerm>>, members: Vec< (String, GraphSearch<M>) > },
    MapEnum { enum_repr: Option<Box<TypeTerm>>, variants: Vec< (String, GraphSearch<M>) > }
}

#[derive(Debug)]
pub enum SolvedStep<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Prim { σs: HashMapSubst, m: M },
    MapSeq { seq_repr: Option<Box<TypeTerm>>, item: MorphismInstance<M> },
    MapStruct { struct_repr: Option<Box<TypeTerm>>, members: Vec< (String, MorphismInstance<M>) > },
    MapEnum { enum_repr: Option<Box<TypeTerm>>, variants: Vec< (String, MorphismInstance<M>) > }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct SearchNodePtr<M:Morphism+Clone>( pub Arc<RwLock<SearchNode<M>>> );

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M:Morphism+Clone> PartialEq for SearchNodePtr<M> {
    fn eq(&self, other: &Self) -> bool {
        let locked_self = self.0.read().unwrap();
        let locked_other = other.0.read().unwrap();

        (other.0.get_weight() + locked_other.est_remain)
                == (self.0.get_weight() + locked_self.est_remain)
    }
}

impl<M:Morphism+Clone> Eq for SearchNodePtr<M> {

}

impl<M:Morphism+Clone> Ord for SearchNodePtr<M> {
    fn cmp(&self, other: &Self) -> Ordering {
        let locked_self = self.0.read().unwrap();
        let locked_other = other.0.read().unwrap();

        (other.0.get_weight() + locked_other.est_remain)
            .cmp(
                &(self.0.get_weight() + locked_self.est_remain)
            )
    }
}

impl<M:Morphism+Clone> PartialOrd for SearchNodePtr<M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait SearchNodeExt<M: Morphism+Clone> {
   // fn specialize(&self, σ: HashMapSubst) -> Arc<RwLock<SearchNode<M>>>;
    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>>;

    fn chain(&self, id: u64, ψ: TypeTerm, Γ: &ContextPtr, σs: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>>;
    fn map_seq(&self, id: u64, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>>;
    fn map_struct(&self, id: u64, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;
    fn map_enum(&self, id: u64, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;

    fn advance(&self, base: &MorphismBase<M>) -> Result<bool, GraphSearchError>;
    fn to_morphism_instance(&self) -> Option< MorphismInstance<M> >;

    fn is_ready(&self) -> bool;
    fn get_weight(&self) -> u64;
    fn get_weight_step(&self) -> u64;
    fn est_remain(&self, goal: &MorphismType) -> u64;
    fn get_type(&self) -> MorphismType;

    fn creates_loop(&self) -> bool;
}

impl<M: Morphism+Clone> SearchNodeExt<M> for Arc<RwLock<SearchNode<M>>> {
    fn get_weight_step(&self) -> u64 {
        match &self.read().unwrap().step {
            Step::Id { τ } => 0,
            Step::Prim { σs, m } => 10,
            Step::MapSeq { seq_repr, item } => item.best_path_weight(),
            Step::MapStruct { struct_repr, members } => members.iter().map(|(_,g)| g.best_path_weight() ).sum(),
            Step::MapEnum { enum_repr, variants } => variants.iter().map(|(_,g)| g.best_path_weight() ).max().unwrap_or(0),
        }
    }

    fn get_weight(&self) -> u64 {
        self.read().unwrap().weight
        + self.get_weight_step()
    }

    /*
     * for node `search_node` , calculate the estimated cost for completing
     * the path to fulfill the morphism type `goal`
     */
    fn est_remain(&self, goal: &MorphismType) -> u64 {
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: goal.src_type.clone(),
            dst_type: self.get_type().src_type.clone()
        }.estimated_cost()
        +
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: self.get_type().dst_type.clone(),
            dst_type: goal.dst_type.clone()
        }.estimated_cost()
    }

    fn get_type(&self) -> MorphismType {
        let s = self.read().unwrap();
        MorphismType {
            Γ: vec![],//s.ctx.get_Γ(),
            bounds: Vec::new(),
            src_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.src_type.clone() ]).normalize(),
            dst_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.dst_type.clone() ]).normalize(),
        }.apply_subst(&s.ctx)
    }

    // tell if this sub-search already has a solution
    fn is_ready(&self) -> bool {
        let n = self.read().unwrap();
        match &n.step {
            Step::Id { τ } => true,
            Step::Prim { σs, m } => true,
            Step::MapSeq { seq_repr, item } => {
                item.get_solution().is_some()
            }
            Step::MapStruct { struct_repr, members } => {
                members.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
            Step::MapEnum { enum_repr, variants } => {
                variants.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
        }
    }

    fn creates_loop(&self) -> bool {
        //eprintln!("-- is loop ? --");
        let mut end_type = self.get_type().dst_type;//self.read().unwrap().ty.dst_type.clone();
        let mut cur_node = self.read().unwrap().pred.clone();
        while let Some(n) = cur_node {
            let s = &n.read().unwrap().step;
            let prev_type = match s {
                Step::Id { τ:_ } |
                Step::Prim { σs:_, m:_ } => { n.get_type().dst_type },
                Step::MapSeq { seq_repr, item } => {
                    TypeTerm::Seq { seq_repr: seq_repr.clone(), item: Box::new(item.goal.dst_type.clone()) }
                },
                Step::MapStruct { struct_repr, members } => {
                    TypeTerm::Struct { struct_repr: struct_repr.clone(),
                        members: members.iter().map(
                            |(symbol,search)| StructMember {
                                symbol: symbol.clone(),
                                ty: search.goal.dst_type.clone()
                            }).collect()
                    }
                },
                Step::MapEnum { enum_repr, variants } => {
                    TypeTerm::Enum { enum_repr: enum_repr.clone(),
                            variants: variants.iter().map(
                                |(symbol,search)| EnumVariant {
                                    symbol: symbol.clone(),
                                    ty: search.goal.dst_type.clone()
                                }).collect()
                    }
                },
            }.normalize();

            let ctx = self.read().unwrap().ctx.clone();
            //eprintln!("check for loop: {} =?= {}", prev_type.pretty(&mut ctx.clone(), 0), end_type.pretty(&mut ctx.clone(), 0));

            if prev_type == end_type
            //if unify(&prev_type, &end_type).is_ok()
            {
                //eprintln!("--- loop ---");
                return true;
            }

            cur_node = n.read().unwrap().pred.clone();
        }
        //eprintln!("--- no loop --");

        false
    }

    fn advance(&self, base: &MorphismBase<M>) -> Result<bool, GraphSearchError> {
        let mut n = self.write().unwrap();
        match &mut n.step {
            Step::MapSeq { seq_repr, item } => {
                match item.advance(base) {
                    GraphSearchState::Solved(item_morph) => {
                        n.ty = MorphismType {
                            Γ: Vec::new(),
                            bounds: Vec::new(),
                            src_type: TypeTerm::Seq { seq_repr: seq_repr.clone(), item: Box::new(item_morph.get_type().src_type) },
                            dst_type: TypeTerm::Seq { seq_repr: seq_repr.clone(), item: Box::new(item_morph.get_type().dst_type) },
                        };
                        Ok(true)
                    }
                    GraphSearchState::Continue => Ok(false),
                    GraphSearchState::Err(err) => Err(err)
                }
            }
            Step::MapStruct { struct_repr, members } => {
                for (symbol, sub_search) in members.iter_mut() {
                    if sub_search.get_solution().is_none() {
                        match sub_search.advance(base) {
                            GraphSearchState::Solved(_) => {
                                return Ok(false);
                            },
                            GraphSearchState::Continue => { return Ok(false); },
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
                return Ok(true);
            }
            Step::MapEnum { enum_repr, variants } => {
                for (symbol, sub_search) in variants.iter_mut() {
                    if sub_search.get_solution().is_none() {
                        match sub_search.advance(base) {
                            GraphSearchState::Solved(_) => {
                                return Ok(false);
                            },
                            GraphSearchState::Continue => { return Ok(false); },
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
                return Ok(true);
            }
            _ => Ok(true)
        }
    }

    fn chain(&self, id: u64, ψ: TypeTerm, ctx_inst: &ContextPtr, σs: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>> {
        //eprintln!("CHAIN with σs: ={:?}, Γ={}", σs, ctx_inst.pretty());
        let mut src_type = self.get_type().src_type;
        let mut dst_type = m.get_type().dst_type;
        dst_type.apply_subst(&σs);

        let n = Arc::new(RwLock::new(SearchNode {
            id,
            ctx: ctx_inst.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            est_remain: 0,
            ty: MorphismType { Γ: ctx_inst.get_Γ(), bounds: Vec::new(), src_type, dst_type },
            step: Step::Prim{ σs, m },
            ψ,
        }));

        n
    }

    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>> {
        self.write().unwrap().ψ = ψ;//TypeTerm::Ladder(vec![ ψ, oldψ.clone() ]).normalize();
        self.clone()
    }

    fn map_seq(&self, id: u64, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>> {
        let seq_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Seq { seq_repr, item } => { seq_repr.clone() }
            _ => unreachable!()
        };

        let ctx = self.read().unwrap().ctx.clone().scope(AddressingMode::StackUp);

        Arc::new(RwLock::new(SearchNode {
            id,
            ctx: self.read().unwrap().ctx.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            est_remain: 0,
            ty: MorphismType {
                    Γ: Vec::new(),
                    bounds: Vec::new(),
                    src_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(), item: Box::new(goal.src_type.clone()) },
                    dst_type: TypeTerm::Seq{ seq_repr: seq_repr.clone(), item: Box::new(goal.dst_type.clone()) }
                },
            step: Step::MapSeq { seq_repr, item: GraphSearch::new(ctx, goal) },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_struct(&self, id: u64, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {
        let struct_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Struct { struct_repr, members } => { struct_repr.clone() }
            _ => unreachable!()
        };

        Arc::new(RwLock::new(SearchNode {
            id,
            ctx: self.read().unwrap().ctx.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            est_remain: 0,
            ty: MorphismType {
                Γ: Vec::new(),
                bounds:Vec::new(),
                src_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Struct { struct_repr: struct_repr.clone(), members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapStruct {
                struct_repr,
                members: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(self.read().unwrap().ctx.scope(AddressingMode::StackUp), goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_enum(&self, id: u64, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {
        let enum_repr = match self.read().unwrap().ty.dst_type.get_floor_type().1 {
            TypeTerm::Enum { enum_repr, variants } => { enum_repr.clone() }
            _ => unreachable!()
        };

        Arc::new(RwLock::new(SearchNode {
            id,
            ctx: self.read().unwrap().ctx.clone(),
            pred: Some(self.clone()),
            weight: self.get_weight(),
            est_remain: 0,
            ty: MorphismType {
                Γ: Vec::new(),
                bounds: Vec::new(),
                src_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Enum { enum_repr: enum_repr.clone(), variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapEnum { enum_repr, variants: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(self.read().unwrap().ctx.scope(AddressingMode::StackUp), goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }


    fn to_morphism_instance(&self) -> Option< MorphismInstance<M> > {
        let mut steps = Vec::new();
        let mut cur_node = Some(self.clone());

        while let Some(n) = cur_node {
            let n = n.read().unwrap();
            steps.push((n.ctx.clone(), n.ψ.clone(),
                match &n.step {
                    Step::Id { τ } => SolvedStep::Id { τ: τ.clone() },
                    Step::Prim { σs, m } => SolvedStep::Prim { σs: σs.clone(), m: m.clone() },
                    Step::MapSeq { seq_repr, item } => {
                        let item = item.get_solution().unwrap();
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

            cur_node = n.pred.clone();
        }

        steps.reverse();

        let mut begin = TypeTerm::unit();
        let mut path = Vec::new();
        let mut σ = HashMapSubst::new();

        for (n_ctx, ψ, s) in steps {

            for (v,t) in n_ctx.0.read().unwrap().σ.iter() {
                σ.insert(*v,t.clone());
            }
            σ.saturate();

            match s {
                SolvedStep::Id { τ } => {
                    begin = τ.clone();
                }
                SolvedStep::Prim{ σs, m } => {
                    let mut m = MorphismInstance::Primitive { σs: σs.clone(), m:m.clone() };
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

        if σ.len() > 0 {
            Some(
                MorphismInstance::Specialize {
                    σ,
                    m: Box::new(MorphismInstance::from_chain(begin, &path))
                }
            )
        } else {
            Some(MorphismInstance::from_chain(begin, &path))
        }
    }
}
