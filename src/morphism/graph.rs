use {
    crate::{
        morphism::DecomposedMorphismType, EnumVariant, HashMapSubst, Morphism, MorphismBase, MorphismInstance, MorphismType, StructMember, SubstitutionMut, TypeDict, TypeTerm
    },
    std::{collections::HashMap, sync::{Arc,RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct MorphismGraph<M: Morphism+Clone> {
    solved_paths: HashMap< MorphismType, MorphismInstance<M> >,
    base: MorphismBase<M>
}

pub struct GraphSearch<M: Morphism+Clone> {
    goal: MorphismType,
    solution: Option< MorphismInstance<M> >,
    explore_queue: Vec< Arc<RwLock<SearchNode<M>>> >,

    skip_preview: bool
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

/// represents a partial path during search in the morphism graph
pub struct SearchNode<M: Morphism+Clone> {
    /// predecessor node
    pred: Option< Arc<RwLock< SearchNode<M> >> >,

    /// (measured) weight of the preceding path
    weight: u64,

    ty: MorphismType,

    /// the advancement over pred
    step: Step<M>,
    ψ: TypeTerm,
}

pub enum Step<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Inst{ m: MorphismInstance<M> },
    //Sub{ ψ: TypeTerm },
    Specialize { σ: HashMapSubst },
    MapSeq { item: GraphSearch<M> },
    MapStruct { members: Vec< (String, GraphSearch<M>) > },
    MapEnum { variants: Vec< (String, GraphSearch<M>) > }
}

pub enum SolvedStep<M: Morphism+Clone> {
    Id { τ: TypeTerm },
    Inst{ m: MorphismInstance<M> },
    Specialize { σ: HashMapSubst },
    MapSeq { item: MorphismInstance<M> },
    MapStruct { members: Vec< (String, MorphismInstance<M>) > },
    MapEnum { variants: Vec< (String, MorphismInstance<M>) > }
}


//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait SearchNodeExt<M: Morphism+Clone> {
    fn specialize(&self, σ: HashMapSubst) -> Arc<RwLock<SearchNode<M>>>;
    fn chain(&self, ψ: TypeTerm, σ: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>>;
    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>>;
    fn map_seq(&self, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>>;
    fn map_struct(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;
    fn map_enum(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>>;

    fn advance(&self, base: &MorphismBase<M>, dict: &mut impl TypeDict) -> Result<bool, GraphSearchError>;
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
            Step::Inst { m } => m.get_weight(),
            Step::Specialize { σ } => 0,
            Step::MapSeq { item } => item.best_path_weight(),
            Step::MapStruct { members } => members.iter().map(|(_,g)| g.best_path_weight() ).sum(),
            Step::MapEnum { variants } => variants.iter().map(|(_,g)| g.best_path_weight() ).max().unwrap_or(0),
        }
    }

    fn get_type(&self) -> MorphismType {
        let s = self.read().unwrap();
        MorphismType {
            bounds: Vec::new(),
            src_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.src_type.clone() ]).normalize(),
            dst_type: TypeTerm::Ladder(vec![ s.ψ.clone(), s.ty.dst_type.clone() ]).normalize(),
        }
    }

    // tell if this sub-search already has a solution
    fn is_ready(&self) -> bool {
        let n = self.read().unwrap();
        match &n.step {
            Step::Id { τ } => true,
            //Step::Sub { ψ } => n.pred.as_ref().unwrap().is_ready(),
            Step::Specialize { σ } => true,
            Step::MapSeq { item } => {
                item.get_solution().is_some()
            }
            Step::MapStruct { members } => {
                members.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
            Step::MapEnum { variants } => {
                variants.iter().map(|(s,g)| g.get_solution().is_some()).min().unwrap_or(true)
            }
            Step::Inst { m } => true
        }
    }

    fn creates_loop(&self) -> bool {
        /*
        let mut cur_node = self.read().unwrap().pred.clone();
        while let Some(n) = cur_node {
            if n.get_type().dst_type == self.get_type().dst_type {
                return true;
            }

            cur_node = n.read().unwrap().pred.clone();
        }
        */

        false
    }

    fn advance(&self, base: &MorphismBase<M>, dict: &mut impl TypeDict) -> Result<bool, GraphSearchError> {
        let mut n = self.write().unwrap();
        match &mut n.step {
            Step::MapSeq { item } => {
                //eprintln!("advance seq-map");
                match item.advance(base, dict) {
                    GraphSearchState::Solved(item_morph) => {
                        //eprintln!("Sequence-Map Sub Graph Solved!!");
                        n.ty = MorphismType {
                            bounds: Vec::new(),
                            src_type: TypeTerm::Seq { seq_repr: None, items: vec![ item_morph.get_type().src_type ] },
                            dst_type: TypeTerm::Seq { seq_repr: None, items: vec![ item_morph.get_type().dst_type ] },
                        };
                        Ok(false)
                    }
                    GraphSearchState::Continue => Ok(true),
                    GraphSearchState::Err(err) => Err(err)
                }
            }
            Step::MapStruct { members } => {
                for (symbol, sub_search) in members.iter_mut() {
                    if sub_search.get_solution().is_none() {
                        match sub_search.advance(base, dict) {
                            GraphSearchState::Solved(_) => {
                                return Ok(true);
                            },
                            GraphSearchState::Continue => { return Ok(true); },
                            GraphSearchState::Err(err) => { return Err(err); }
                        }
                    }
                }
                return Ok(false);
            }
            Step::MapEnum { variants } => {
                todo!()
            }
            _ => Ok(false)
        }
    }

    fn specialize(&self, σ: HashMapSubst) -> Arc<RwLock<SearchNode<M>>> {

        let src_type=  self.get_type().src_type;
        let dst_type= self.get_type().dst_type;

        let σ = σ.filter(|(v,t)| src_type.contains_var(*v) || dst_type.contains_var(*v));
        if σ.is_empty() {
            self.clone()
        } else {
            Arc::new(RwLock::new(SearchNode {
                pred: Some(self.clone()),
                weight: self.get_weight(),
                ty: MorphismType {
                        bounds: Vec::new(),
                        src_type: self.get_type().src_type.apply_subst(&σ).clone().normalize(),
                        dst_type: self.get_type().dst_type.apply_subst(&σ).clone().normalize()
                    },
                step: Step::Specialize { σ },
                ψ: self.read().unwrap().ψ.clone()
            }))
        }
    }

    fn chain(&self, ψ: TypeTerm, σ: HashMapSubst, m: M) -> Arc<RwLock<SearchNode<M>>> {
        let m = MorphismInstance::Primitive { m: m.clone() };

        let mut parent = self.clone();

        {
            let mut σ = σ.clone().filter(|(v,t)|
                self.get_type().dst_type.contains_var(*v)
            );
            parent = parent.specialize(σ);
        }

        let mut σ_src = σ.clone().filter(|(v,t)|
                m.get_type().src_type.contains_var(*v) &&
                !self.get_type().dst_type.contains_var(*v)
            );

        let mut σ_dst = σ.clone().filter(|(v,t)|
                m.get_type().dst_type.contains_var(*v)
        );

        let mut n = Arc::new(RwLock::new(SearchNode {
            pred: Some(parent),
            weight: self.get_weight(),
            ty: MorphismType {
                bounds: Vec::new(),
                src_type: self.get_type().src_type,
                dst_type: m.get_type().dst_type
            }.apply_subst(&σ_src),
            step: Step::Inst{
                m: if σ_src.is_empty() {
                    m
                } else {
                    MorphismInstance::Specialize { σ: σ_src, m: Box::new(m) }
                }
            },
            ψ: TypeTerm::unit(),
        }));
        n.set_sub(ψ);
        n = n.specialize(σ_dst);
        n
    }

    fn set_sub(&self, ψ: TypeTerm) -> Arc<RwLock<SearchNode<M>>> {
        let oldψ = &mut self.write().unwrap().ψ;
        *oldψ = TypeTerm::Ladder(vec![ ψ, oldψ.clone() ]).normalize();
        self.clone()
    }

    fn map_seq(&self, goal: MorphismType) -> Arc<RwLock<SearchNode<M>>> {
        Arc::new(RwLock::new(SearchNode {
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                    bounds: Vec::new(),
                    src_type: TypeTerm::Seq{ seq_repr: None, items: vec![goal.src_type.clone()] },
                    dst_type: TypeTerm::Seq{ seq_repr: None, items: vec![goal.src_type.clone()] }
                },
            step: Step::MapSeq { item: GraphSearch::new(goal) },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_struct(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {
        Arc::new(RwLock::new(SearchNode {
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                bounds:Vec::new(),
                src_type: TypeTerm::Struct { struct_repr: None, members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Struct { struct_repr: None, members: goals.iter().map(|(s,t)| StructMember{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapStruct { members: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn map_enum(&self, goals: Vec<(String, MorphismType)>) -> Arc<RwLock<SearchNode<M>>> {
        Arc::new(RwLock::new(SearchNode {
            pred: Some(self.clone()),
            weight: self.get_weight(),
            ty: MorphismType {
                bounds: Vec::new(),
                src_type: TypeTerm::Enum { enum_repr: None, variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.src_type.clone() }).collect() },
                dst_type: TypeTerm::Enum { enum_repr: None, variants: goals.iter().map(|(s,t)| EnumVariant{ symbol: s.clone(), ty: t.dst_type.clone() }).collect() }
            },
            step: Step::MapEnum { variants: goals.into_iter().map(|(name,goal)| (name, GraphSearch::new(goal))).collect() },
            ψ: self.read().unwrap().ψ.clone()
        }))
    }

    fn to_morphism_instance(&self) -> Option< MorphismInstance<M> > {
        let mut steps = Vec::new();
        let mut cur_node = Some(self.clone());
        while let Some(n) = cur_node {
            let n = n.read().unwrap();
            steps.push((n.ψ.clone(),
                match &n.step {
                    Step::Id { τ } => SolvedStep::Id { τ: τ.clone() },
                    Step::Inst { m } => SolvedStep::Inst { m: m.clone() },
                    Step::Specialize { σ } => SolvedStep::Specialize { σ: σ.clone() },
                    Step::MapSeq { item } => SolvedStep::MapSeq { item: item.get_solution().unwrap() },
                    Step::MapStruct { members } => SolvedStep::MapStruct { members: members.iter().map(|(n,m)| (n.clone(), m.get_solution().unwrap())).collect() },
                    Step::MapEnum { variants } => SolvedStep::MapEnum { variants: variants.iter().map(|(n,m)| (n.clone(), m.get_solution().unwrap())).collect() },
                }));

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
                    begin = τ.clone(); },
                SolvedStep::Inst{ m } => {
                    //eprintln!("to_morph_instance: Inst {:?} -- {:?}", ψ, m.get_type());
                    let mut m = m.clone();
                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m.clone());
                },
                SolvedStep::Specialize { σ } => {
                    //eprintln!("to_morph_instance: Specialize {:?}", σ);
                    if path.len() > 0 && !σ.is_empty() {
                        let m = MorphismInstance::from_chain(begin.clone(), &path);
                        path = vec![
                            MorphismInstance::Specialize { σ: σ.clone(), m: Box::new(m) }
                        ];
                    }
                }
                SolvedStep::MapSeq { item } => {
                    let mut m = MorphismInstance::MapSeq {
                        seq_repr: None,
                        item_morph: Box::new(item)
                    };

                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
                SolvedStep::MapStruct { members } => {
                    let mut m = MorphismInstance::MapStruct {
                        src_struct_repr: None,
                        dst_struct_repr: None,
                        member_morph: members
                    };
                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
                SolvedStep::MapEnum { variants } => {
                    let mut m = MorphismInstance::MapEnum {
                        enum_repr: None,
                        variant_morph: variants
                    };
                    if ! ψ.is_empty() {
                        m = MorphismInstance::Sub { ψ, m: Box::new(m) };
                    }
                    path.push(m);
                }
            }
        }

        Some(MorphismInstance::from_chain(begin, &path))
    }
}




//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: Morphism+Clone> MorphismGraph<M> {
    pub fn new(base: MorphismBase<M>) -> Self {
        MorphismGraph {
            solved_paths: HashMap::new(),
            base
        }
    }

    pub fn search(&self, goal: MorphismType, dict: &mut impl TypeDict) -> Result<
        MorphismInstance<M>,
        GraphSearchError
    > {
        let mut search = GraphSearch::<M>::new(goal);
        loop {
            match search.advance(&self.base, dict) {
                GraphSearchState::Solved(m) => { return Ok(m); }
                GraphSearchState::Continue => { continue; }
                GraphSearchState::Err(err) => { return Err(err); }
            }
        }
    }
}

impl<M: Morphism+Clone> GraphSearch<M> {
    pub fn new(goal: MorphismType) -> Self {
        GraphSearch {
            goal: goal.clone(),
            solution: None,
            explore_queue: vec![
                Arc::new(RwLock::new(SearchNode {
                    pred: None,
                    weight: 0,
                    ty: MorphismType {
                        bounds: Vec::new(),
                        src_type: goal.src_type.clone(),
                        dst_type: goal.src_type.clone()
                    },
                    step: Step::Id { τ: goal.src_type.clone() },
                    ψ: TypeTerm::unit()
                }))
            ],
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

    pub fn est_remain(goal: &MorphismType, search_node: &Arc<RwLock<SearchNode<M>>>) -> u64 {
        MorphismType {
            bounds: Vec::new(),
            src_type: goal.src_type.clone(),
            dst_type: search_node.get_type().src_type.clone()
        }.estimated_cost()
        +
        MorphismType {
            bounds: Vec::new(),
            src_type: search_node.get_type().dst_type.clone(),
            dst_type: goal.dst_type.clone()
        }.estimated_cost()
    }

    pub fn choose_next_node(&mut self, dict: &mut impl TypeDict) -> Option<Arc<RwLock<SearchNode<M>>>> {
        let goal= self.goal.clone();
        self.explore_queue.sort_by(
            |a,b| {
                (Self::est_remain(&goal, b) + b.get_weight() )
                    .cmp(
                        &(Self::est_remain(&goal, a) + a.get_weight())
                    )
            }
        );

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
                    n.get_type().src_type.pretty(dict, 0),
                    n.get_type().dst_type.pretty(dict, 0));
            }
        } else {
            self.skip_preview = false;
        }

        self.explore_queue.pop()
    }

    pub fn advance(&mut self, base: &MorphismBase<M>, dict: &mut impl TypeDict) -> GraphSearchState<M> {
        if let Some(node) = self.choose_next_node(dict) {
            match node.advance(base, dict) {
                Ok(_) => {
                    if ! node.is_ready() {
                        if ! node.creates_loop() {
                            self.skip_preview = true;
                            self.explore_queue.push(node);
                        }
                        return GraphSearchState::Continue;
                    } else {
                        let w = node.to_morphism_instance().unwrap().get_weight();
                        eprintln!("set Weight of complex morph to {}", w);
                        node.write().unwrap().weight = w;
                    }
                }
                Err(err) => {
                    return GraphSearchState::Err(err);
                }
            }

            if node.creates_loop() {
                eprintln!("Creates loop.");
                return GraphSearchState::Continue;
            }

            /* 1. Check if goal is already reached by the current path */
            if let Ok((_ψ, σ)) = crate::constraint_system::subtype_unify( &node.get_type().dst_type, &self.goal.dst_type ) {
                /* found path */
                self.solution = Some(
                    if σ.is_empty() {
                        node.to_morphism_instance().unwrap()
                    } else {
                        node
                            .specialize( σ.filter_morphtype(&node.get_type()) )
                            .to_morphism_instance()
                            .unwrap()
                    }
                );

                return GraphSearchState::Solved(self.get_solution().unwrap());
            }

            let mut decompositions = base.enum_complex_morphisms(&node.get_type().dst_type);
            if let Some(d) = base.morphism_decomposition(&node.get_type().dst_type, &self.goal.dst_type) {
                decompositions.push(d);
            }

            eprintln!("{} decompositions", decompositions.len());

            let mut done = Vec::new();
            for (ψ,decomposition) in decompositions {
                if ! done.contains(&(ψ.clone(),decomposition.clone())) {
                    self.explore_queue.push(
                        match &decomposition {
                            DecomposedMorphismType::SeqMap { item } => { node.map_seq( item.clone() ) },
                            DecomposedMorphismType::StructMap { members } => { node.map_struct(members.clone()) },
                            DecomposedMorphismType::EnumMap { variants } => { node.map_enum(variants.clone()) },
                        }.set_sub(ψ.clone())
                    );
                    done.push((ψ, decomposition));
                }else {
                    //eprintln!("avoid duplicate decomposition");
                }
            }

            /* 2. Try to advance current path */
            for (ψ,σ,m) in base.enum_morphisms_from(&node.get_type().dst_type) {
                eprintln!("add direct path");
                self.explore_queue.push( node.chain(ψ,σ,m) );
            }

            GraphSearchState::Continue
        } else {
            GraphSearchState::Err(GraphSearchError::NoMorphismFound)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
