use {
    crate::{
        morphism::{Morphism, MorphismInstance, MorphismType}, HashMapSubst, StructMember, TypeDict, TypeTerm
    }, std::io::Write
};

pub trait MorphBase<
    Morph: Morphism + Clone,
    Weight: Eq + Ord + Default
> {
    fn get_morphisms(&self, halo_key: &TypeTerm) -> Vec<MorphismInstance<Morph>> {
        vec![]
    }

    fn heuristic(&self, t: &MorphismType) -> Weight {
        Weight::default()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DecomposedMorphismType {
    SeqMap { item: MorphismType },
    StructMap { members: Vec<(String, MorphismType)> },
    EnumMap { variants: Vec<(String, MorphismType)> }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone)]
pub struct MorphismBase<M: Morphism + Clone> {
    morphisms: Vec< M >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: Morphism + Clone> MorphismBase<M> {
    pub fn new() -> Self {
        MorphismBase {
            morphisms: Vec::new()
        }
    }

    pub fn add_morphism(&mut self, m: M) {
        self.morphisms.push( m );
    }

    /*
       given a morphism type (src/dst types),
       try to match their outer structure (Struct/Seq/Map)
       and spawn a GraphSearch for each component
     */
    pub fn morphism_decomposition(&self, src_type: &TypeTerm, dst_type: &TypeTerm) ->
        Option< (TypeTerm, DecomposedMorphismType) >
    {
        let (src_ψ, src_floor) = src_type.get_floor_type();
        let (dst_ψ, dst_floor) = dst_type.get_floor_type();

        if !dst_ψ.is_empty() {
            if !crate::constraint_system::subtype_unify(&src_ψ, &dst_ψ).is_ok() {
                return None;
            }
        }

        match (src_floor, dst_floor) {
            (TypeTerm::Struct{ struct_repr: struct_repr_lhs, members: members_lhs},
                TypeTerm::Struct { struct_repr: struct_repr_rhs, members: members_rhs })
            => {
                // todo: optimization: check if struct repr match

                let mut member_morph_types = Vec::new();
                let mut failed = false;
                let mut necessary = false;

                for StructMember{ symbol: symbol_rhs, ty: ty_rhs } in members_rhs.iter() {
                    let mut found_src_member = false;
                    for StructMember{ symbol: symbol_lhs, ty: ty_lhs } in members_lhs.iter() {
                        if symbol_rhs == symbol_lhs {
                            found_src_member = true;
                            member_morph_types.push((symbol_rhs.clone(), MorphismType {
                                bounds: Vec::new(),
                                src_type: ty_lhs.clone(), dst_type: ty_rhs.clone() }));
                            if ty_lhs != ty_rhs {
                                necessary = true;
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
                    Some((src_ψ, DecomposedMorphismType::StructMap {
                        members: member_morph_types
                    }))
                } else {
                    None
                }
            }


            (TypeTerm::Seq{ seq_repr: seq_repr_lhs, items: items_lhs },
                TypeTerm::Seq{ seq_repr: _seq_rerpr_rhs, items: items_rhs })
            => {
                for (ty_lhs, ty_rhs) in items_lhs.iter().zip(items_rhs.iter()) {
                    return Some((src_ψ, DecomposedMorphismType::SeqMap {
                        item: MorphismType{
                            bounds: Vec::new(),
                            src_type: ty_lhs.clone(), dst_type: ty_rhs.clone() }
                    }));
                }
                None
            }

            (TypeTerm::Enum { enum_repr: enum_repr_lhs, variants: variants_lhs },
                TypeTerm::Enum { enum_repr: enum_repr_rhs, variants: variants_rhs }
            ) => {
                todo!()
            }

            _ => {
                None
            }
        }
    }

    pub fn enum_morphisms_from(&self, src_type: &TypeTerm) -> Vec< (TypeTerm, HashMapSubst, M) > {
        let mut morphs = Vec::new();

        for m in self.morphisms.iter() {
            let m_src_type = m.get_type().src_type.normalize();
            let m_dst_type = m.get_type().dst_type.normalize();

            /* 1. primitive morphisms */

            // check if the given source type is compatible with the
            // morphisms source type,
            // i.e. check if `src_type` is a subtype of `m_src_type`
            if let Ok((ψ, σ)) = crate::constraint_system::subtype_unify(src_type, &m_src_type) {
                morphs.push((ψ, σ, m.clone()));
            }
        }

        morphs
    }

    pub fn enum_complex_morphisms(&self, src_type: &TypeTerm) -> Vec<(TypeTerm, DecomposedMorphismType)> {
        let mut morphs = Vec::new();
        for m in self.morphisms.iter() {
            let m_src_type = m.get_type().src_type.normalize();

            /* 2. check complex types */
            if let Some(decomposition) = self.morphism_decomposition(src_type, &m_src_type) {
                morphs.push(decomposition);
            }
        }
        morphs
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
