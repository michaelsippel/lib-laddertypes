use {
    crate::{
        morphism_path_sugared::{SugaredMorphismPath, SugaredShortestPathProblem}, morphism_sugared::{MorphismInstance2, SugaredMorphism, SugaredMorphismType}, sugar::SugaredTypeTerm, SugaredStructMember, TypeDict, TypeID, TypeTerm
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct SugaredMorphismBase<M: SugaredMorphism + Clone> {
    morphisms: Vec< M >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: SugaredMorphism + Clone> SugaredMorphismBase<M> {
    pub fn new() -> Self {
        SugaredMorphismBase {
            morphisms: Vec::new()
        }
    }

    pub fn add_morphism(&mut self, m: M) {
        self.morphisms.push( m );
    }

    pub fn get_morphism_instance(&self, ty: &SugaredMorphismType) -> Option<MorphismInstance2<M>> {
        if let Some(path) = SugaredShortestPathProblem::new(self, ty.clone()).solve() {
            if path.len() == 1 {
                Some(path[0].clone())
            } else {
                Some(MorphismInstance2::Chain { path })
            }
        } else {
            None
        }
    }

    pub fn complex_morphism_decomposition(&self, src_type: &SugaredTypeTerm, dst_type: &SugaredTypeTerm) -> Option< MorphismInstance2<M> > {
        let (src_ψ, src_floor) = src_type.get_floor_type();
        let (dst_ψ, dst_floor) = dst_type.get_floor_type();

        if !dst_ψ.is_empty() {
            if !crate::unification_sugared::subtype_unify(&src_ψ, &dst_ψ).is_ok() {
                return None;
            }
        }

        match (src_floor, dst_floor) {
            (SugaredTypeTerm::Struct{ struct_repr: struct_repr_lhs, members: members_lhs},
                SugaredTypeTerm::Struct { struct_repr: struct_repr_rhs, members: members_rhs })
            => {
                // todo: optimization: check if struct repr match

                let mut member_morph = Vec::new();
                let mut failed = false;
                let mut necessary = false;

                for SugaredStructMember{ symbol: symbol_rhs, ty: ty_rhs } in members_rhs.iter() {
                    let mut found_src_member = false;
                    for SugaredStructMember{ symbol: symbol_lhs, ty: ty_lhs } in members_lhs.iter() {
                        if symbol_rhs == symbol_lhs {
                            found_src_member = true;

                            if let Some(mm) = self.get_morphism_instance(&SugaredMorphismType { src_type: ty_lhs.clone(), dst_type: ty_rhs.clone() }) {
                                if ty_lhs != ty_rhs {
                                    necessary = true;
                                }
                                member_morph.push((symbol_lhs.clone(), mm))
                            } else {
                                failed = true;
                            }
                            break;
                        }
                    }

                    // member of rhs not found in lhs
                    if ! found_src_member {
                        failed = true;
                        break;
                    }
                }

                if ! failed && necessary {
                    Some(MorphismInstance2::MapStruct {
                        ψ: src_ψ,
                        src_struct_repr: struct_repr_lhs.clone(),
                        dst_struct_repr: struct_repr_rhs.clone(),
                        member_morph
                    })
                } else {
                    None
                }
            }


            (SugaredTypeTerm::Seq{ seq_repr: seq_repr_lhs, items: items_lhs },
                SugaredTypeTerm::Seq{ seq_repr: _seq_rerpr_rhs, items: items_rhs })
            => {
                //let mut item_morphs = Vec::new();

                for (ty_lhs, ty_rhs) in items_lhs.iter().zip(items_rhs.iter()) {
                    if let Some(item_morph) = self.get_morphism_instance(&SugaredMorphismType{ src_type: ty_lhs.clone(), dst_type: ty_rhs.clone() }) {
                        return Some(MorphismInstance2::MapSeq { ψ: src_ψ, seq_repr: seq_repr_lhs.clone(), item_morph: Box::new(item_morph) });
                    }
                    break;
                }
                None
            }

            _ => {
                None
            }
        }
    }

    pub fn enum_morphisms_from(&self, src_type: &SugaredTypeTerm) -> Vec< MorphismInstance2<M> > {
        let mut morphs = Vec::new();

        for m in self.morphisms.iter() {
            let m_src_type = m.get_type().src_type;
            let m_dst_type = m.get_type().dst_type;

            /* 1. primitive morphisms */

            // check if the given start type is compatible with the
            // morphisms source type,
            // i.e. check if `src_type` is a subtype of `m_src_type`
            if let Ok((ψ, σ)) = crate::unification_sugared::subtype_unify(src_type, &m_src_type) {
                let morph_inst = MorphismInstance2::Primitive { ψ, σ, morph: m.clone() };
                //eprintln!("..found direct morph to {:?}", morph_inst.get_type().dst_type);
                morphs.push(morph_inst);
            }

            /* 2. check complex types */
            else if let Some(complex_morph) = self.complex_morphism_decomposition(src_type, &m_src_type) {
                //eprintln!("found complex morph to {:?}", complex_morph.get_type().dst_type);
                morphs.push(complex_morph);
            }
        }

        morphs
    }

}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
