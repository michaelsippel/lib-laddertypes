use {
    crate::{
        morphism::{Morphism, MorphismInstance, MorphismType}, Context, ContextPtr, HashMapSubst, LayeredContext, StructMember, TypeDict, TypeTerm
    }, std::{arch::x86_64::_MM_ROUND_NEAREST, collections::HashMap, io::Write, sync::{Arc, RwLock}}
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
    Γ: ContextPtr,
    morphisms: Vec< M >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl<M: Morphism + Clone> MorphismBase<M> {
    pub fn new(Γ: ContextPtr) -> Self {
        MorphismBase {
            Γ,
            morphisms: Vec::new()
        }
    }

    pub fn ctx(&self) -> ContextPtr {
        self.Γ.clone()
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

                            // todo: check if member-morph-type is parallel

                            member_morph_types.push((symbol_rhs.clone(), MorphismType {
                                bounds: Vec::new(),
                                src_type: ty_lhs.clone(), dst_type: ty_rhs.clone()
                            }));

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

    pub fn enum_morphisms_from(&self, Γ0: &ContextPtr, src_type: &TypeTerm) -> Vec< (TypeTerm, ContextPtr, HashMapSubst, M) > {
        let mut morphs = Vec::new();

        for m in self.morphisms.iter() {
            let mut m_src_type = m.get_type().src_type.normalize();
            let mut m_dst_type = m.get_type().dst_type.normalize();

            let Γ = Γ0.scope();
            let σs = Γ.shift_variables(&m.ctx());
            m_src_type.apply_subst(&σs);
            m_dst_type.apply_subst(&σs);

            let mut src_type = src_type.clone();
            src_type.apply_subst(&Γ.shift_from_parent());


            // check if the given source type is compatible with the
            // morphisms source type,
            // i.e. check if `src_type` is a subtype of `m_src_type`
            if let Ok((ψ, σ)) = crate::constraint_system::subtype_unify(&src_type, &m_src_type) {
                for (v,t) in σ.iter() {
                    Γ.bind(*v, t.clone()).expect("cant bind variable");
                }
                morphs.push((ψ, Γ, σs, m.clone()));
            }
        }

        morphs
    }

    pub fn enum_complex_morphisms(&self, Γ0: &ContextPtr, src_type: &TypeTerm) -> Vec<(TypeTerm, ContextPtr, HashMapSubst, DecomposedMorphismType)> {
        let mut morphs = Vec::<(TypeTerm, ContextPtr, HashMapSubst, DecomposedMorphismType)>::new();
        for m in self.morphisms.iter() {
            let mut src_type = src_type.clone();
            let mut m_src_type = m.get_type().src_type.normalize();

            let Γ = Γ0.scope();
            let σs = Γ.shift_variables(&m.ctx());
            m_src_type.apply_subst(&σs);

            src_type.apply_subst(&Γ.shift_from_parent());

            /* 2. check complex types */
            if let Some((ψ,decomposition)) = self.morphism_decomposition(&src_type, &m_src_type) {
                morphs.push((ψ,Γ,σs,decomposition));
            }
        }
        morphs
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
