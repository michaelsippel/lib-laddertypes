use {
    crate::{
        dict::*,
        morphism::{Morphism, MorphismType, MorphismInstance},
        morphism_base::MorphismBase,
        substitution::Substitution,
        term::*, desugared_term::*,
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct MorphismPath<M: Morphism + Clone> {
    pub weight: u64,
    pub cur_type: TypeTerm,
    pub morphisms: Vec< MorphismInstance<M> >
}


impl<M: Morphism+Clone> MorphismPath<M> {
    fn apply_subst(&mut self, σ: &std::collections::HashMap<TypeID, TypeTerm>) {
        for m in self.morphisms.iter_mut() {
            m.apply_subst(σ);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct ShortestPathProblem<'a, M: Morphism + Clone> {
    pub morphism_base: &'a MorphismBase<M>,
    pub goal: TypeTerm,
    queue: Vec< MorphismPath<M> >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<'a, M:Morphism+Clone> ShortestPathProblem<'a, M> {
    pub fn new(morphism_base: &'a MorphismBase<M>, ty: MorphismType) -> Self {
        ShortestPathProblem {
            morphism_base,
            queue: vec![
                MorphismPath::<M> { weight: 0, cur_type: ty.src_type, morphisms: vec![] }
            ],
            goal: ty.dst_type
        }
    }

    pub fn advance(&mut self, prev_path: &MorphismPath<M>, morph_inst: MorphismInstance<M>) {
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

    pub fn solve(&mut self) -> Option< Vec<MorphismInstance<M>> > {
        while ! self.queue.is_empty() {
            /* take the shortest partial path and try to advance it by one step */
            self.queue.sort_by( |p1,p2| p2.weight.cmp(&p1.weight));
            if let Some(mut cur_path) = self.queue.pop() {

                /* 1. Check if goal is already reached by the current path */
                if let Ok((_ψ, σ)) = crate::constraint_system::subtype_unify( &cur_path.cur_type, &self.goal ) {
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
