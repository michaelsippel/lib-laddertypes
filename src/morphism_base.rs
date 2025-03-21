use {
    crate::{
        subtype_unify, sugar::SugaredTypeTerm, unification::UnificationProblem, unparser::*, TypeDict, TypeID, TypeTerm,
        morphism::{MorphismType, Morphism, MorphismInstance}
    },
    std::{collections::HashMap, u64}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct MorphismBase<M: Morphism + Clone> {
    morphisms: Vec< M >,
    seq_types: Vec< TypeTerm >
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

    pub fn enum_direct_morphisms(&self, src_type: &TypeTerm)
    -> Vec< MorphismInstance<M> >
    {
        let mut dst_types = Vec::new();
        for m in self.morphisms.iter() {
            if let Ok((halo, σ)) = crate::unification::subtype_unify(
                &src_type.clone().param_normalize(),
                &m.get_type().src_type.param_normalize(),
            ) {
                dst_types.push(MorphismInstance{ halo, m: m.clone(), σ });
            }
        }
        dst_types
    }

    pub fn enum_map_morphisms(&self, src_type: &TypeTerm)
    -> Vec< MorphismInstance<M> > {
        let src_type = src_type.clone().param_normalize();
        let mut dst_types = Vec::new();

        // Check if we have a List type, and if so, see what the Item type is
        // TODO: function for generating fresh variables
        let item_variable = TypeID::Var(800);

        for seq_type in self.seq_types.iter() {
            if let Ok((halo, σ)) = crate::unification::subtype_unify(
                &src_type,
                &TypeTerm::App(vec![
                    seq_type.clone(),
                    TypeTerm::TypeID(item_variable)
                ])
            ) {
                let src_item_type = σ.get(&item_variable).expect("var not in unificator").clone();
                for item_morph_inst in self.enum_morphisms( &src_item_type ) {

                    let mut dst_halo_ladder = vec![ halo.clone() ];
                    if item_morph_inst.halo != TypeTerm::unit() {
                        dst_halo_ladder.push(
                            TypeTerm::App(vec![
                                seq_type.clone().get_lnf_vec().first().unwrap().clone(),
                                item_morph_inst.halo.clone()
                            ]));
                    }

                    if let Some( map_morph ) = item_morph_inst.m.map_morphism( seq_type.clone() ) {
                        dst_types.push(
                            MorphismInstance {
                                halo: TypeTerm::Ladder(dst_halo_ladder).strip().param_normalize(),
                                m: map_morph,
                                σ: item_morph_inst.σ
                            }
                        );
                    } else {
                        eprintln!("could not get map morphism");
                    }
                }
            }
        }

        dst_types
    }

    pub fn enum_morphisms(&self, src_type: &TypeTerm) -> Vec< MorphismInstance<M> > {
        let mut dst_types = Vec::new();
        dst_types.append(&mut self.enum_direct_morphisms(src_type));
        dst_types.append(&mut self.enum_map_morphisms(src_type));
        dst_types
    }

    pub fn find_direct_morphism(&self,
        ty: &MorphismType,
        dict: &mut impl TypeDict
    ) -> Option< MorphismInstance<M> > {
        eprintln!("find direct morph");
        for m in self.morphisms.iter() {
            let ty = ty.clone().normalize();
            let morph_type = m.get_type().normalize();

            eprintln!("find direct morph:\n   {}  <=   {}",
                            dict.unparse(&ty.src_type), dict.unparse(&morph_type.src_type),
                        );

            if let Ok((halo, σ)) = subtype_unify(&ty.src_type, &morph_type.src_type) {
                eprintln!("halo: {}", dict.unparse(&halo));

                let dst_type = TypeTerm::Ladder(vec![
                    halo.clone(),
                    morph_type.dst_type.clone()
                ]).normalize().param_normalize();

                eprintln!("----------->   {}  <=   {}",
                    dict.unparse(&dst_type), dict.unparse(&ty.dst_type)
                );

                if let Ok((halo2, σ2)) = subtype_unify(&dst_type, &ty.dst_type) {
                    eprintln!("match. halo2 = {}", dict.unparse(&halo2));
                    return Some(MorphismInstance {
                        m: m.clone(),
                        halo,
                        σ,
                    });
                }
            }
        }
        None
    }

    pub fn find_map_morphism(&self, ty: &MorphismType, dict: &mut impl TypeDict) -> Option< MorphismInstance<M> > {
        for seq_type in self.seq_types.iter() {
            if let Ok((halos, σ)) = UnificationProblem::new_sub(vec![
                (ty.src_type.clone().param_normalize(),
                    TypeTerm::App(vec![ seq_type.clone(), TypeTerm::TypeID(TypeID::Var(100)) ])),

                (TypeTerm::App(vec![ seq_type.clone(), TypeTerm::TypeID(TypeID::Var(101)) ]),
                    ty.dst_type.clone().param_normalize()),
            ]).solve() {
                // TODO: use real fresh variable names
                let item_morph_type = MorphismType {
                    src_type: σ.get(&TypeID::Var(100)).unwrap().clone(),
                    dst_type: σ.get(&TypeID::Var(101)).unwrap().clone(),
                }.normalize();

                //eprintln!("Map Morph: try to find item-morph with type {:?}", item_morph_type);
                if let Some(item_morph_inst) = self.find_morphism( &item_morph_type, dict ) {
                    if let Some( list_morph ) = item_morph_inst.m.map_morphism( seq_type.clone() ) {
                        return Some( MorphismInstance {
                            m: list_morph,
                            σ,
                            halo: halos[0].clone()
                        } );
                    }
                }
            }
        }

        None
    }

    pub fn find_morphism(&self, ty: &MorphismType,
        dict: &mut impl TypeDict
    )
    -> Option< MorphismInstance<M> > {
        if let Some(m) = self.find_direct_morphism(ty, dict) {
            return Some(m);
        }
        if let Some(m) = self.find_map_morphism(ty, dict) {
            return Some(m);
        }
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
