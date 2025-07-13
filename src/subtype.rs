use crate::term::DesugaredTypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl DesugaredTypeTerm {
    // returns ladder-step of first match and provided representation-type
    pub fn is_semantic_subtype_of(&self, expected_type: &DesugaredTypeTerm) -> Option<(usize, DesugaredTypeTerm)> {
        let provided_lnf = self.clone().get_lnf_vec();
        let expected_lnf = expected_type.clone().get_lnf_vec();

        for i in 0..provided_lnf.len() {
            if provided_lnf[i] == expected_lnf[0] {
                return Some((i, DesugaredTypeTerm::Ladder(
                    provided_lnf[i..].into_iter().cloned().collect()
                )))
            }
        }

        None
    }

    pub fn is_syntactic_subtype_of(&self, expected_type: &DesugaredTypeTerm) -> Result<usize, (usize, usize)> {
        if let Some((first_match, provided_type)) = self.is_semantic_subtype_of( expected_type ) {
            let provided_lnf = provided_type.get_lnf_vec();
            let expected_lnf = expected_type.clone().get_lnf_vec();

            for i in 0 .. usize::min( provided_lnf.len(), expected_lnf.len() ) {
                if provided_lnf[i] != expected_lnf[i] {
                    return Err((first_match, first_match+i))
                }
            }

            Ok(first_match)
        } else {
            Err((0,0))
        }
    }


    // supertype analogs

    pub fn is_semantic_supertype_of(&self, t: &DesugaredTypeTerm) -> Option<(usize, DesugaredTypeTerm)> {
        t.is_semantic_subtype_of(self)
    }

    pub fn is_syntactic_supertype_of(&self, t: &DesugaredTypeTerm) -> Result<usize, (usize, usize)> {
        t.is_syntactic_subtype_of(self)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

use crate::sugar::*;

impl TypeTerm {
    pub fn is_compatible(&self, supertype: TypeTerm) -> bool {
        match (self, supertype) {
            (TypeTerm::TypeID(idl), TypeTerm::TypeID(idr)) => {
                if *idl == idr {
                    true
                } else {
                    false
                }
            }


            (TypeTerm::Ladder(l_rungs), TypeTerm::Ladder(r_rungs)) => {
                false
            }

            _ => {
                false
            }
        }
    }
}
