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
        morphism::DecomposedMorphismType, search_node::{SearchNode, SearchNodeExt, Step}, AddressingMode, Context, ContextPtr, EnumVariant, HashMapSubst, LayeredContext, Morphism, MorphismBase, MorphismInstance, MorphismType, StructMember, SubstitutionMut, TypeDict, TypeTerm
    },
    std::{collections::HashMap, ops::Deref, sync::{Arc,RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct MorphismGraph<M: Morphism+Clone> {
    solved_paths: HashMap< MorphismType, MorphismInstance<M> >,
    base: MorphismBase<M>
}

pub struct GraphSearch<M: Morphism+Clone> {
    Γ: ContextPtr,
    pub goal: MorphismType,
    solution: Option< MorphismInstance<M> >,
    explore_queue: Vec< Arc<RwLock<SearchNode<M>>> >,

    pub history: Vec< Arc<RwLock<SearchNode<M>>> >,

    skip_preview: bool,
    id_count: u64
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphSearchState<M: Morphism+Clone> {
    Solved( MorphismInstance<M> ),
    Continue,
    Err( GraphSearchError )
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphSearchError {
    NoMorphismFound
}


//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: Morphism+Clone> MorphismGraph<M> {
    pub fn new(base: MorphismBase<M>) -> Self {
        MorphismGraph {
            solved_paths: HashMap::new(),
            base
        }
    }

    pub fn search(&self, goal: MorphismType) -> (
        Result<
            MorphismInstance<M>,
            GraphSearchError
        >,
        GraphSearch<M>
    )
    {
        let Γ = self.base.ctx().scope(AddressingMode::StackUp);
        eprintln!("Start search (Γ={})", Γ.get_ctxname());
        let mut search = GraphSearch::<M>::new(Γ, goal);
        loop {
            match search.advance(&self.base) {
                GraphSearchState::Solved(m) => { return (Ok(m), search); }
                GraphSearchState::Continue => { continue; }
                GraphSearchState::Err(err) => { return (Err(err), search); }
            }
        }
    }
}

impl<M: Morphism+Clone> GraphSearch<M> {
    pub fn new(ctx: ContextPtr, goal: MorphismType) -> Self {

        let start_node = Arc::new(RwLock::new(SearchNode {
            id: 0,
            ctx: ctx.clone(),
            pred: None,
            weight: 0,
            ty: MorphismType {
                Γ: Vec::new(),
                bounds: Vec::new(),
                src_type: goal.src_type.clone(),
                dst_type: goal.src_type.clone()
            },
            step: Step::Id { τ: goal.src_type.clone() },
            ψ: TypeTerm::unit()
        }));

        GraphSearch {
            id_count: 1,
            goal: goal,
            solution: None,
            Γ: ctx,
            history: vec![ start_node.clone() ],
            explore_queue: vec![ start_node ],
            skip_preview: false
        }
    }

    pub fn get_solution(&self) -> Option< MorphismInstance<M> > {
        self.solution.clone()
    }

    pub fn best_path_weight(&self) -> u64 {
        if let Some(best) = self.explore_queue.last() {
            best.get_weight()
        } else {
            0
        }
    }

    /*
     * for node `search_node` , calculate the estimated cost for completing
     * the path to fulfill the morphism type `goal`
     */
    pub fn est_remain(goal: &MorphismType, search_node: &Arc<RwLock<SearchNode<M>>>) -> u64 {
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: goal.src_type.clone(),
            dst_type: search_node.get_type().src_type.clone()
        }.estimated_cost()
        +
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: search_node.get_type().dst_type.clone(),
            dst_type: goal.dst_type.clone()
        }.estimated_cost()
    }

    /*
     * consider the nodes in `self.explore_queue` and take the most promising node
     */
    pub fn choose_next_node(&mut self, dict: &mut impl TypeDict) -> Option<Arc<RwLock<SearchNode<M>>>> {
        let goal= self.goal.clone();

        /* sort all nodes by descending weight whereby we use the sum of the
         * already manifested cost of the taken path
         * plus the estimated remaining cost to complete the path
         */
        self.explore_queue.sort_by(
            |a,b| {
                (Self::est_remain(&goal, b) + b.get_weight() )
                    .cmp(
                        &(Self::est_remain(&goal, a) + a.get_weight())
                    )
            }
        );

        /*
        if !self.skip_preview {
            eprintln!("===== TOP 5 PATHS =====\nGoal:\n {} -> {}",
                goal.src_type.pretty(dict, 0),
                goal.dst_type.pretty(dict, 0)
            );
            for i in 1 ..= usize::min(self.explore_queue.len(), 5) {
                let n = &self.explore_queue[self.explore_queue.len() - i];
                eprintln!("[[ {} ]] (weight: {} + est remain: {}) ---  {} --> {}", i,
                    n.get_weight(),
                    Self::est_remain(&goal, &n),
                    n.get_type().src_type.pretty(&mut n.read().unwrap().Γ.clone(), 0),
                    n.get_type().dst_type.pretty(&mut n.read().unwrap().Γ.clone(), 0));
            }
        } else {
            self.skip_preview = false;
        }
        */

        self.explore_queue.pop()
    }

    pub fn add_search_node(&mut self, node: Arc<RwLock<SearchNode<M>>>) {
        if ! node.creates_loop() {
            self.explore_queue.push(node.clone());
            self.history.push(node);
        }
    }

    /*
     * take the most promising node and iterate its search by one step
     */
    pub fn advance(&mut self, base: &MorphismBase<M>) -> GraphSearchState<M> {
        if let Some(node) = self.choose_next_node(&mut self.Γ.clone()) {
            let mut nctx = node.read().unwrap().ctx.clone();

            /*
             * in case this node contains a sub-search graph,
             * advance it first
             */
            match node.advance(base) {
                Ok(true) => {
                    /* sub search solved */
                    assert!( node.is_ready() );
                    let w = node.to_morphism_instance().unwrap().get_weight();
                    //eprintln!("set Weight of complex morph to {}", w);
                    node.write().unwrap().weight = w;
                }
                Ok(false) => {
                    self.skip_preview = true;

                    // sub graph needs further exploration, add it back to the queue
                    self.explore_queue.push(node);
                    return GraphSearchState::Continue;
                }
                Err(err) => {
                    // sub graph failed, dont add it back to the queue
                    return GraphSearchState::Continue;
                }
            }

            /* 1. Check if goal is already reached by the current path */
            if let Ok((_ψ, σ)) = crate::constraint_system::subtype_unify( &node.get_type().dst_type, &self.goal.dst_type ) {
                for (v,t) in σ.into_iter() {
                    node.read().unwrap().ctx.bind(v, t).expect("cant bind");
                }

                /* found path */
                self.solution = node.to_morphism_instance();
                return GraphSearchState::Solved(self.get_solution().unwrap());
            }

            let mut decompositions = base.enum_complex_morphisms(&node.read().unwrap().ctx, &node.get_type().dst_type);
            if let Some((ψ,d)) = base.morphism_decomposition(&node.get_type().dst_type, &self.goal.dst_type) {
                decompositions.push((ψ,node.read().unwrap().ctx.clone(),HashMap::new(),d));
            }

            let mut done = Vec::new();
            for (ψ,Γ,σs,decomposition) in decompositions {
                if !decomposition.is_trivial() {
                    if ! done.contains(&(ψ.clone(),σs.clone(),decomposition.clone())) {
                        let id = self.id_count;
                        self.id_count += 1;
                        let mut new_node =
                            match &decomposition {
                                DecomposedMorphismType::SeqMap { item } => { node.map_seq( id ,item.clone() ) },
                                DecomposedMorphismType::StructMap { members } => { node.map_struct(id,members.clone()) },
                                DecomposedMorphismType::EnumMap { variants } => { node.map_enum(id,variants.clone()) },
                            }.set_sub(ψ.clone());

                        new_node.write().unwrap().ctx = Γ;

                        self.add_search_node(new_node);
                        done.push((ψ, σs, decomposition));
                    }
                }
            }

            /* 2. Try to advance current path */
            for (ψ,Γ,σs,m) in base.enum_morphisms_from(&node.read().unwrap().ctx, &node.get_type().dst_type) {
                let id = self.id_count;
                self.id_count += 1;
                self.add_search_node( node.chain(id,ψ.normalize(),&Γ,σs,m) );
            }

            GraphSearchState::Continue
        } else {
            GraphSearchState::Err(GraphSearchError::NoMorphismFound)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
