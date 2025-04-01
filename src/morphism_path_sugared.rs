use {
    crate::{
        dict::*,
        morphism_sugared::{SugaredMorphism, SugaredMorphismType, MorphismInstance2},
        morphism_base_sugared::SugaredMorphismBase,
        substitution_sugared::SugaredSubstitution,
        sugar::*, term::*,
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct SugaredMorphismPath<M: SugaredMorphism + Clone> {
    pub weight: u64,
    pub cur_type: SugaredTypeTerm,
    pub morphisms: Vec< MorphismInstance2<M> >
}


impl<M: SugaredMorphism+Clone> SugaredMorphismPath<M> {
    fn apply_subst(&mut self, σ: &std::collections::HashMap<TypeID, SugaredTypeTerm>) {
        for m in self.morphisms.iter_mut() {
            m.apply_subst(σ);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct SugaredShortestPathProblem<'a, M: SugaredMorphism + Clone> {
    pub morphism_base: &'a SugaredMorphismBase<M>,
    pub goal: SugaredTypeTerm,
    queue: Vec< SugaredMorphismPath<M> >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<'a, M:SugaredMorphism+Clone> SugaredShortestPathProblem<'a, M> {
    pub fn new(morphism_base: &'a SugaredMorphismBase<M>, ty: SugaredMorphismType) -> Self {
        SugaredShortestPathProblem {
            morphism_base,
            queue: vec![
                SugaredMorphismPath::<M> { weight: 0, cur_type: ty.src_type, morphisms: vec![] }
            ],
            goal: ty.dst_type
        }
    }

    pub fn advance(&mut self, prev_path: &SugaredMorphismPath<M>, morph_inst: MorphismInstance2<M>) {
        let dst_type = morph_inst.get_type().dst_type;
        //eprintln!("try morph to {:?}", dst_type.clone());//.sugar(type_dict).pretty(type_dict, 0));

        let mut creates_loop = false;

        let mut new_path = prev_path.clone();
        new_path.apply_subst(&morph_inst.get_subst());
        for m in new_path.morphisms.iter() {
            if m.get_type().src_type == dst_type {
                creates_loop = true;
                break;
            }
        }

        if ! creates_loop {
            new_path.weight += 1;//next_morph_inst.get_weight();
            new_path.cur_type = dst_type;

            new_path.morphisms.push(morph_inst);
            self.queue.push(new_path);
        }
    }

    pub fn solve(&mut self) -> Option< Vec<MorphismInstance2<M>> > {
        while ! self.queue.is_empty() {
            /* take the shortest partial path and try to advance it by one step */
            self.queue.sort_by( |p1,p2| p2.weight.cmp(&p1.weight));
            if let Some(mut cur_path) = self.queue.pop() {

                /* 1. Check if goal is already reached by the current path */
                if let Ok((_ψ, σ)) = crate::unification_sugared::subtype_unify( &cur_path.cur_type, &self.goal ) {
                    /* found path,
                     * now apply substitution and trim to variables in terms of each step
                     */
                    cur_path.apply_subst(&σ);
                    return Some(cur_path.morphisms);
                }

                /* 2. Try to advance current path */
                else if let Some(complex_morph) =
                    self.morphism_base.complex_morphism_decomposition( &cur_path.cur_type, &self.goal )
                {
                    self.advance(&cur_path, complex_morph);
                }

                for next_morph_inst in self.morphism_base.enum_morphisms_from(&cur_path.cur_type) {
                    self.advance(&cur_path, next_morph_inst);
                }
            }
        }
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
