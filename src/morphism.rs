use {
    crate::{
        TypeTerm, TypeID,
        unification::UnificationProblem,
    },
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MorphismType {
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm,
}

pub trait Morphism : Sized {
    fn get_type(&self) -> MorphismType;
    fn map_morphism(&self, seq_type: TypeTerm) -> Option< Self >;

    fn weight(&self) -> u64 {
        1
    }
}

#[derive(Clone)]
pub struct MorphismBase<M: Morphism + Clone> {
    morphisms: Vec< M >,
    seq_types: Vec< TypeTerm >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl MorphismType {
    pub fn normalize(self) -> Self {
        MorphismType {
            src_type: self.src_type.normalize(),
            dst_type: self.dst_type.normalize()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: Morphism + Clone> MorphismBase<M> {
    pub fn new(seq_types: Vec<TypeTerm>) -> Self {
        MorphismBase {
            morphisms: Vec::new(),
            seq_types
        }
    }

    pub fn add_morphism(&mut self, m: M) {
        self.morphisms.push( m );
    }

    pub fn enum_morphisms(&self, src_type: &TypeTerm)
    -> Vec< (HashMap<TypeID, TypeTerm>, TypeTerm) >
    {
        let mut dst_types = Vec::new();

        // first enumerate all "direct" morphisms,
        for m in self.morphisms.iter() {
            if let Ok(σ) = crate::unification::unify(
                &m.get_type().src_type,
                &src_type.clone().normalize()
            ) {
                let dst_type =
                    m.get_type().dst_type.clone()
                    .apply_substitution( &σ )
                    .clone();

                dst_types.push( (σ, dst_type) );
            }
        }

        // ..then all "list-map" morphisms.
        // Check if we have a List type, and if so, see what the Item type is

        // TODO: function for generating fresh variables
        let item_variable = TypeID::Var(100);

        for seq_type in self.seq_types.iter() {
        if let Ok(σ) = crate::unification::unify(
            &TypeTerm::App(vec![
                seq_type.clone(),
                TypeTerm::TypeID(item_variable)
            ]),
            &src_type.clone().param_normalize(),
        ) {
            let src_item_type = σ.get(&item_variable).unwrap().clone();

            for (γ, dst_item_type) in self.enum_morphisms( &src_item_type ) {
                let dst_type =
                        TypeTerm::App(vec![
                            seq_type.clone(),
                            dst_item_type.clone()
                                .apply_substitution(&γ).clone()
                        ]).normalize();

                dst_types.push( (γ.clone(), dst_type) );
            }
        }
        }

        dst_types
    }

    pub fn enum_morphisms_with_subtyping(
        &self,
        src_type: &TypeTerm,
    ) -> Vec<(TypeTerm, TypeTerm, HashMap<TypeID, TypeTerm>)> {
        let mut src_lnf = src_type.clone().get_lnf_vec();
        let mut halo_lnf = vec![];
        let mut dst_types = Vec::new();

        while src_lnf.len() > 0 {
            let src_type = TypeTerm::Ladder(src_lnf.clone());
            let halo_type = TypeTerm::Ladder(halo_lnf.clone());

            for (σ, t) in self.enum_morphisms(&src_type) {
                dst_types.push((
                    halo_type
                        .clone()
                        .apply_substitution(&σ)
                        .clone(),
                    t.clone().apply_substitution(&σ).clone(),
                    σ,
                ));
            }

            // continue with next supertype
            halo_lnf.push(src_lnf.remove(0));
        }

        dst_types
    }

    /* try to find shortest morphism-path for a given type
     */
    pub fn find_morphism_path(&self, ty: MorphismType)
    -> Option< Vec<TypeTerm> >
    {
        let ty = ty.normalize();

        let mut queue = vec![
            (0, vec![ ty.src_type.clone().normalize() ])
        ];

        while ! queue.is_empty() {
            queue.sort_by( |&(w1,_),&(w2,_)| w2.cmp(&w1));

            if let Some((current_weight, current_path)) = queue.pop() {
                let current_type = current_path.last().unwrap();
                for (h, t, σp) in self.enum_morphisms_with_subtyping(&current_type) {
                    let tt = TypeTerm::Ladder(vec![h, t]).normalize();

                    if !current_path.contains(&tt) {
                        let unification_result = crate::unification::unify(&tt, &ty.dst_type);
                        let morphism_weight = 1;
                        /*self.find_morphism( &tt ).unwrap().0.get_weight()*/

                        let new_weight = current_weight + morphism_weight;
                        let mut new_path = current_path.clone();

                        new_path.push(tt);

                        for n in new_path.iter_mut() {
                            n.apply_substitution(&σp);
                        }

                        if let Ok(σ) = unification_result {
                            for n in new_path.iter_mut() {
                                n.apply_substitution(&σ);
                            }
                            return Some(new_path);
                        } else {
                            queue.push((new_weight, new_path));
                        }
                    }
                }
            }
        }

        None
    }


    pub fn find_morphism(&self, ty: &MorphismType)
    -> Option< ( M, HashMap<TypeID, TypeTerm> ) > {

        // try to find primitive morphism
        for m in self.morphisms.iter() {
            let unification_problem = UnificationProblem::new_sub(
                vec![
                    ( ty.src_type.clone().normalize(), m.get_type().src_type.clone() ),
                    ( ty.dst_type.clone().normalize(), m.get_type().dst_type.clone() )
                ]
            );

            let unification_result = unification_problem.solve();
            if let Ok((ψ,σ)) = unification_result {
                return Some((m.clone(), σ));
            }
        }

        // try list-map morphism
        for seq_type in self.seq_types.iter() {
            eprintln!("try seq type {:?}", seq_type);

            eprintln!("");

            if let Ok((ψ,σ)) = UnificationProblem::new_sub(vec![
                (ty.src_type.clone().param_normalize(),
                    TypeTerm::App(vec![ seq_type.clone(), TypeTerm::TypeID(TypeID::Var(100)) ])),
                (ty.dst_type.clone().param_normalize(),
                    TypeTerm::App(vec![ seq_type.clone(), TypeTerm::TypeID(TypeID::Var(101)) ])),
            ]).solve() {
                // TODO: use real fresh variable names
                let item_morph_type = MorphismType {
                    src_type: σ.get(&TypeID::Var(100)).unwrap().clone(),
                    dst_type: σ.get(&TypeID::Var(101)).unwrap().clone(),
                }.normalize();

                if let Some((m, σ)) = self.find_morphism( &item_morph_type ) {
                    if let Some(list_morph) = m.map_morphism( seq_type.clone() ) {
                        return Some( (list_morph, σ) );
                    }
                }
            }
        }

        None
    }

    pub fn find_morphism_with_subtyping(&self, ty: &MorphismType)
    -> Option<( M, TypeTerm, HashMap<TypeID, TypeTerm> )> {
        let mut src_lnf = ty.src_type.clone().get_lnf_vec();
        let mut dst_lnf = ty.dst_type.clone().get_lnf_vec();
        let mut halo = vec![];

        while src_lnf.len() > 0 && dst_lnf.len() > 0 {
            if let Some((m, σ)) = self.find_morphism(&MorphismType{
                src_type: TypeTerm::Ladder(src_lnf.clone()),
                dst_type: TypeTerm::Ladder(dst_lnf.clone())
            }) {
                halo.push(src_lnf.get(0).unwrap().clone());
                return Some((m,
                    TypeTerm::Ladder(halo).apply_substitution(&σ).clone(),
                    σ));
            } else {
                if src_lnf[0] == dst_lnf[0] {
                    src_lnf.remove(0);
                    halo.push(dst_lnf.remove(0));
                } else {
                    return None;
                }
            }
        }

        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
