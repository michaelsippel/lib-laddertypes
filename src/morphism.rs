use {
    crate::{
        TypeTerm, TypeID,
        unification::UnificationProblem,
    },
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct MorphismType {
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm,
}

pub struct MorphismBase<Morphism: Clone> {
    morphisms: Vec< (MorphismType, Morphism) >,
    list_typeid: TypeID
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl MorphismType {
    fn normalize(self) -> Self {
        MorphismType {
            src_type: self.src_type.normalize(),
            dst_type: self.dst_type.normalize()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<Morphism: Clone> MorphismBase<Morphism> {
    pub fn new() -> Self {
        MorphismBase {
            morphisms: Vec::new(),

            // FIXME: magic number
            list_typeid: TypeID::Fun(10)
        }
    }

    pub fn add_morphism(&mut self, morph_type: MorphismType, morphism: Morphism) {
        self.morphisms.push( (morph_type.normalize(), morphism) );
    }

    pub fn enum_morphisms(&self, src_type: &TypeTerm)
    -> Vec< (HashMap<TypeID, TypeTerm>, TypeTerm) >
    {
        let mut dst_types = Vec::new();

        // first enumerate all "direct" morphisms,
        for (ty,m) in self.morphisms.iter() {
            if let Ok(σ) = crate::unification::unify(
                &ty.src_type,
                &src_type.clone().normalize()
            ) {
                let dst_type =
                    ty.dst_type.clone()
                        .apply_substitution(&σ)
                        .clone();

                dst_types.push( (σ, dst_type) );
            }
        }

        // ..then all "list-map" morphisms.
        // Check if we have a List type, and if so, see what the Item type is

        // TODO: function for generating fresh variables
        let item_variable = TypeID::Var(100);

        if let Ok(σ) = crate::unification::unify(
            &TypeTerm::App(vec![
                TypeTerm::TypeID(self.list_typeid),
                TypeTerm::TypeID(item_variable)
            ]),
            &src_type.clone().param_normalize(),
        ) {
            let src_item_type = σ.get(&item_variable).unwrap().clone();

            for (γ, dst_item_type) in self.enum_morphisms( &src_item_type ) {
                let dst_type =
                        TypeTerm::App(vec![
                            TypeTerm::TypeID(self.list_typeid),
                            dst_item_type.clone()
                                .apply_substitution(&γ).clone()
                        ]).normalize();

                dst_types.push( (γ.clone(), dst_type) );
            }
        }

        dst_types
    }

    pub fn enum_morphisms_with_subtyping(&self, src_type: &TypeTerm)
    -> Vec< (TypeTerm, TypeTerm) >
    {
        let mut src_lnf = src_type.clone().get_lnf_vec();
        let mut halo_lnf = vec![];
        let mut dst_types = Vec::new();

        while src_lnf.len() > 0 {
            let src_type = TypeTerm::Ladder( src_lnf.clone() );
            let halo_type = TypeTerm::Ladder( halo_lnf.clone() );

            for (σ, t) in self.enum_morphisms( &src_type ) {
                dst_types.push(
                    (halo_type.clone()
                        .apply_substitution(&σ).clone(),
                    t.clone()
                        .apply_substitution(&σ).clone()
                    )
                );
            }

            // continue with next supertype
            halo_lnf.push(src_lnf.remove(0));
        }

        dst_types
    }

    /* performs DFS to find a morphism-path for a given type
     * will return the first matching path, not the shortest
     */
    pub fn find_morphism_path(&self, ty: MorphismType)
    -> Option< Vec<TypeTerm> >
    {
        let ty = ty.normalize();
        let mut visited = Vec::new();
        let mut queue = vec![
            vec![ ty.src_type.clone().normalize() ]
        ];

        while let Some(current_path) = queue.pop() {
            let current_type = current_path.last().unwrap();

            if ! visited.contains( current_type ) {
                visited.push( current_type.clone() );

                for (h, t) in self.enum_morphisms_with_subtyping(&current_type) {
                    let tt = TypeTerm::Ladder( vec![ h, t ] ).normalize();

                    if ! visited.contains( &tt ) {
                        let unification_result = crate::unification::unify(&tt, &ty.dst_type);
                        let mut new_path = current_path.clone();

                        new_path.push( tt );

                        if let Ok(σ) = unification_result {
                            new_path = new_path.into_iter().map(
                                |mut t: TypeTerm| t.apply_substitution(&σ).clone()
                            ).collect::<Vec<TypeTerm>>();

                            return Some(new_path);
                        } else {
                            queue.push( new_path );
                        }
                    }
                }
            }
        }

        None
    }

    pub fn find_morphism(&self, ty: &MorphismType)
    -> Option< Morphism > {

        // TODO

        None
    }

    pub fn find_list_map_morphism(&self, item_ty: &MorphismType)
    -> Option< Morphism > {

        // TODO

        None
    }

    pub fn find_morphism_with_subtyping(&self, ty: &MorphismType)
    -> Option<( Morphism, TypeTerm, HashMap<TypeID, TypeTerm> )> {

        // TODO

        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
