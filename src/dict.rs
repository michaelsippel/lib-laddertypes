use crate::{
    bimap::Bimap,
    sugar::SUGARID_LIMIT
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TypeID {
    Fun(u64),
    Var(u64)
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct TypeDict {
    typenames: Bimap<String, TypeID>,
    type_lit_counter: u64,
    type_var_counter: u64,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeDict {
    pub fn new() -> Self {
        let mut dict = TypeDict {
            typenames: Bimap::new(),
            type_lit_counter: 0,
            type_var_counter: 0,
        };

        dict.add_typename("Seq".into());
        dict.add_typename("Enum".into());
        dict.add_typename("Struct".into());

        assert_eq!( dict.type_lit_counter, SUGARID_LIMIT );

        dict
    }

    pub fn add_varname(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Var(self.type_var_counter);
        self.type_var_counter += 1;
        self.typenames.insert(tn, tyid.clone());
        tyid
    }

    pub fn add_typename(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Fun(self.type_lit_counter);
        self.type_lit_counter += 1;
        self.typenames.insert(tn, tyid.clone());
        tyid
    }

    pub fn add_synonym(&mut self, new: String, old: String) {
        if let Some(tyid) = self.get_typeid(&old) {
            self.typenames.insert(new, tyid);
        }
    }

    pub fn get_typename(&self, tid: &TypeID) -> Option<String> {
        self.typenames.my.get(tid).cloned()
    }

    pub fn get_typeid(&self, tn: &String) -> Option<TypeID> {
        self.typenames.mλ.get(tn).cloned()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
