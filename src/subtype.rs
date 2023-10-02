use crate::term::TypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {    
    // returns ladder-step of first match and provided representation-type
    pub fn is_semantic_subtype_of(&self, expected_type: &TypeTerm) -> Option<(usize, TypeTerm)> {
        let provided_lnf = self.clone().get_lnf_vec();
        let expected_lnf = expected_type.clone().get_lnf_vec();

        for i in 0..provided_lnf.len() {
            if provided_lnf[i] == expected_lnf[0] {
                return Some((i, TypeTerm::Ladder(
                    provided_lnf[i..].into_iter().cloned().collect()
                )))
            }
        }

        None
    }

    pub fn is_syntactic_subtype_of(&self, expected_type: &TypeTerm) -> Result<usize, Option<(usize, usize)>> {
        if let Some((first_match, provided_type)) = self.is_semantic_subtype_of( expected_type ) {
            let provided_lnf = provided_type.get_lnf_vec();
            let expected_lnf = expected_type.clone().get_lnf_vec();

            for i in 0 .. usize::min( provided_lnf.len(), expected_lnf.len() ) {
                if provided_lnf[i] != expected_lnf[i] {
                    return Err(Some((first_match, first_match+i)))
                }
            }

            Ok(first_match)
        } else {
            Err(None)
        }
    }


    // supertype analogs

    pub fn is_semantic_supertype_of(&self, t: &TypeTerm) -> Option<(usize, TypeTerm)> {
        t.is_semantic_subtype_of(self)
    }

    pub fn is_syntactic_supertype_of(&self, t: &TypeTerm) -> Result<usize, Option<(usize, usize)>> {
        t.is_syntactic_subtype_of(self)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
