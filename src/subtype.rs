use crate::term::TypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    pub fn find_semantic_subtype_matches(&self, expected_type: &TypeTerm)
    -> Option<(TypeTerm, TypeTerm, TypeTerm)>
    {
        let provided_lnf = self.clone().get_lnf_vec();
        let expected_lnf = expected_type.clone().get_lnf_vec();

        for i in 0..provided_lnf.len() {
            if provided_lnf[i] == expected_lnf[0] {
                // found first match.
                // now find first mismatch.
                for j in i..usize::min(provided_lnf.len(), i+expected_lnf.len()) {
                    if provided_lnf[j] != expected_lnf[ j-i ] {

                        eprintln!("found match at {}, mismatch at {}", i, j);
                        let syntactic_subladder = TypeTerm::Ladder( provided_lnf[ 0 .. j ].into_iter().cloned().collect() );
                        let provided_reprladder = TypeTerm::Ladder( provided_lnf[ j .. ].into_iter().cloned().collect() );
                        let expected_reprladder = TypeTerm::Ladder( expected_lnf[ j-i .. ].into_iter().cloned().collect() );
                        return Some((syntactic_subladder, provided_reprladder, expected_reprladder));
                    }
                }

                eprintln!("only syntactic subtype");

                // syntactic subtype
                let n = {
                    if provided_lnf.len() + i < expected_lnf.len() {
                        1
                    } else {
                        2
                    }
                };

                let syntactic_subladder = TypeTerm::Ladder( provided_lnf[ 0 .. provided_lnf.len()-1 ].into_iter().cloned().collect() );
                let provided_reprladder = TypeTerm::Ladder( provided_lnf[ provided_lnf.len()-n .. ].into_iter().cloned().collect() );
                let expected_reprladder = TypeTerm::Ladder( expected_lnf[ provided_lnf.len()-n-i .. ].into_iter().cloned().collect() );
                return Some((syntactic_subladder, provided_reprladder, expected_reprladder));
            }
        }

        None
    }
    
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

    pub fn is_syntactic_subtype_of(&self, expected_type: &TypeTerm) -> Result<usize, (usize, usize)> {
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

    pub fn is_semantic_supertype_of(&self, t: &TypeTerm) -> Option<(usize, TypeTerm)> {
        t.is_semantic_subtype_of(self)
    }

    pub fn is_syntactic_supertype_of(&self, t: &TypeTerm) -> Result<usize, (usize, usize)> {
        t.is_syntactic_subtype_of(self)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
