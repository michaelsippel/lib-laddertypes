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
        morphism::DecomposedMorphismType, search_node::{SearchNode, SearchNodeExt, Step}, AddressingMode, Context, ContextPtr, EnumVariant, HashMapSubst, LayeredContext, Morphism, MorphismBase, MorphismInstance, MorphismType, StructMember, SubstitutionMut, TypeDict, TypeTerm,
        search_node::SearchNodePtr
    },
    std::{collections::{HashMap, BinaryHeap}, cmp::Ordering, ops::Deref, sync::{Arc,RwLock}},
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
    explore_queue: BinaryHeap< SearchNodePtr<M> >,
    pub history: Vec< Arc<RwLock<SearchNode<M>>> >,
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
        let mut i = 0;
        let mut search = GraphSearch::<M>::new(Γ, goal);
        loop {
            if i % 100 == 0 {
                eprintln!("{} nodes", search.history.len());
            }
            i += 1;
            match search.advance(&self.base) {
                GraphSearchState::Solved(m) => {
                    eprintln!("finished with {} nodes", search.history.len());
                    return (Ok(m), search); }
                GraphSearchState::Continue => { continue; }
                GraphSearchState::Err(err) => { return (Err(err), search); }
            }
        }
        //(Err(GraphSearchError::NoMorphismFound), search)
    }
}

impl<M: Morphism+Clone> GraphSearch<M> {
    pub fn new(ctx: ContextPtr, goal: MorphismType) -> Self {

        let start_node = Arc::new(RwLock::new(SearchNode {
            id: 0,
            ctx: ctx.clone(),
            pred: None,
            weight: 0,
            est_remain:0,
            ty: MorphismType {
                Γ: Vec::new(),
                bounds: Vec::new(),
                src_type: goal.src_type.clone(),
                dst_type: goal.src_type.clone()
            },
            step: Step::Id { τ: goal.src_type.clone() },
            ψ: TypeTerm::unit()
        }));

        let mut g = GraphSearch {
            id_count: 1,
            goal: goal,
            solution: None,
            Γ: ctx,
            history: Vec::with_capacity(512),
            explore_queue: BinaryHeap::new(),
        };
        g.history.push(start_node.clone());
        g.explore_queue.push(SearchNodePtr(start_node));
        g
    }

    pub fn get_solution(&self) -> Option< MorphismInstance<M> > {
        self.solution.clone()
    }

    pub fn best_path_weight(&self) -> u64 {
        if let Some(best) = self.explore_queue.peek() {
            best.0.get_weight()
        } else {
            0
        }
    }

    /*
     * consider the nodes in `self.explore_queue` and take the most promising node
     */
    pub fn choose_next_node(&mut self, dict: &mut impl TypeDict) -> Option<Arc<RwLock<SearchNode<M>>>> {
        //let goal= self.goal.clone();
        Some(self.explore_queue.pop()?.0)
    }

    pub fn add_search_node(&mut self, node: Arc<RwLock<SearchNode<M>>>) {
        if ! node.creates_loop() {
            let w = node.get_weight();
            let r = node.est_remain(&self.goal);
            node.write().unwrap().weight = w;
            node.write().unwrap().est_remain = r;

            self.explore_queue.push(SearchNodePtr(node.clone()));
        }
    }

    /*
     * take the most promising node and iterate its search by one step
     */
    pub fn advance(&mut self, base: &MorphismBase<M>) -> GraphSearchState<M> {
        if let Some(node) = self.choose_next_node(&mut self.Γ.clone()) {
            let mut nctx = node.read().unwrap().ctx.clone();

            self.history.push(node.clone());

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
                    // sub graph needs further exploration, add it back to the queue
                    // (without history entry)
                    node.write().unwrap().weight = node.get_weight();
                    self.explore_queue.push(SearchNodePtr(node));
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
