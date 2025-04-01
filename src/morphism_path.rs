use {
    crate::{
        morphism::{MorphismType, Morphism, MorphismInstance},
        morphism_base::MorphismBase,
        dict::*,
        term::*
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct MorphismPath<M: Morphism + Clone> {
    pub weight: u64,
    pub cur_type: TypeTerm,
    pub morphisms: Vec< MorphismInstance<M> >
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

    pub fn solve(&mut self) -> Option< Vec<MorphismInstance<M>> > {
        while ! self.queue.is_empty() {
            /* take the shortest partial path and try to advance it by one step */
            self.queue.sort_by( |p1,p2| p2.weight.cmp(&p1.weight));

            if let Some(mut cur_path) = self.queue.pop() {

                /* 1. Check if goal is already reached by the current path */

                if let Ok((ψ, σ)) = crate::unification::subtype_unify( &cur_path.cur_type, &self.goal ) {
                    /* found path,
                     * now apply substitution and trim to variables in terms of each step
                     */
                    for n in cur_path.morphisms.iter_mut() {
                        let src_type = n.m.get_type().src_type;
                        let dst_type = n.m.get_type().dst_type;

                        let mut new_σ = std::collections::HashMap::new();
                        for (k,v) in σ.iter() {
                            if let TypeID::Var(varid) = k {
                                if src_type.contains_var(*varid)
                                || dst_type.contains_var(*varid) {
                                    new_σ.insert(
                                        k.clone(),
                                        v.clone().apply_substitution(&σ).clone().strip()
                                    );
                                }
                            }
                        }
                        for (k,v) in n.σ.iter() {
                            if let TypeID::Var(varid) = k {
                                if src_type.contains_var(*varid)
                                || dst_type.contains_var(*varid) {
                                    new_σ.insert(
                                        k.clone(),
                                        v.clone().apply_substitution(&σ).clone().strip()
                                    );
                                }
                            }
                        }

                        n.halo = n.halo.clone().apply_substitution(&σ).clone().strip().param_normalize();

                        n.σ = new_σ;
                    }

                    return Some(cur_path.morphisms);
                }

                /* 2. Try to advance the path */
                /* 2.1. Direct Morphisms */

                //eprintln!("cur path (w ={}) : @ {:?}", cur_path.weight, cur_path.cur_type);//.clone().sugar(type_dict).pretty(type_dict, 0) );
                for mut next_morph_inst in self.morphism_base.enum_morphisms(&cur_path.cur_type) {
                    let dst_type = next_morph_inst.get_type().dst_type;
//                    eprintln!("try morph to {}", dst_type.clone().sugar(type_dict).pretty(type_dict, 0));

                    let mut creates_loop = false;

                    let mut new_path = cur_path.clone();
                    for n in new_path.morphisms.iter_mut() {
                        let mut new_σ = std::collections::HashMap::new();

                        for (k,v) in next_morph_inst.σ.iter() {
                            new_σ.insert(
                                k.clone(),
                                v.clone().apply_substitution(&next_morph_inst.σ).clone()
                            );
                        }

                        for (k,v) in n.σ.iter() {
                            new_σ.insert(
                                k.clone(),
                                v.clone().apply_substitution(&next_morph_inst.σ).clone()
                            );
                        }

                        n.halo = n.halo.clone().apply_substitution(&next_morph_inst.σ).clone().strip().param_normalize();

                        n.σ = new_σ;
                    }

                    for m in new_path.morphisms.iter() {
                        if m.get_type().src_type == dst_type {
                            creates_loop = true;
                            break;
                        }
                    }

                    if ! creates_loop {
                        new_path.weight += next_morph_inst.m.weight();
                        new_path.cur_type = dst_type;

                        new_path.morphisms.push(next_morph_inst);
                        self.queue.push(new_path);
                    }
                }

                /* 2.2. Try to decompose */
                /* 2.2.1.  Seq - Map */
                /* 2.2.2.  Struct - Map */
                /* 2.2.3.  Enum - Map */

            }
        }
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
