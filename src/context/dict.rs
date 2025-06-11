use crate::bimap::Bimap;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TypeID {
    Fun(u64),
    Var(u64)
}

pub trait TypeDict : Send + Sync {
    fn add_typename(&mut self, tn: &str) -> u64;
    fn get_typeid(&self, tn: &str) -> Option<TypeID>;
    fn get_typename(&self, tid: u64) -> Option<String>;
    fn get_varname(&self, var_id: u64) -> Option<String>;

    fn get_typeid_creat(&mut self, tn: &str) -> TypeID {
        if let Some(id) = self.get_typeid(tn) {
            id
        } else {
            TypeID::Fun(self.add_typename(tn))
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Debug)]
pub struct BimapTypeDict {
    typenames: Bimap<String, TypeID>,
    type_lit_counter: u64,
    type_var_counter: u64,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl BimapTypeDict {
    pub fn new() -> Self {
        BimapTypeDict {
            typenames: Bimap::new(),
            type_lit_counter: 0,
            type_var_counter: 0,
        }
    }

    pub fn insert(&mut self, name: &str, id: TypeID) {
        self.typenames.insert(name.into(), id);
    }
    pub fn add_varname(&mut self, tn: &str) -> u64 {
        let tyid = self.type_var_counter;
        self.type_var_counter += 1;
        self.insert(tn.into(), TypeID::Var(tyid));
        tyid
    }
}

impl TypeDict for BimapTypeDict {
    fn add_typename(&mut self, tn: &str) -> u64 {
        let tyid = self.type_lit_counter;
        self.type_lit_counter += 1;
        self.insert(tn.into(), TypeID::Fun(tyid));
        tyid
    }

    fn get_typename(&self, id: u64) -> Option<String> {
        self.typenames.my.get(&TypeID::Fun(id)).cloned()
    }
    fn get_varname(&self, id: u64) -> Option<String> {
        self.typenames.my.get(&TypeID::Var(id)).cloned()
    }
    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.typenames.mλ.get(tn).cloned()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::{Arc,RwLock};

impl<T: TypeDict> TypeDict for Arc<RwLock<T>> {
    fn add_typename(&mut self, tn: &str) -> u64 {
        self.write().unwrap().add_typename(tn)
    }
    fn get_typename(&self, id: u64)-> Option<String> {
        self.read().unwrap().get_typename(id)
    }
    fn get_varname(&self, id: u64)-> Option<String> {
        self.read().unwrap().get_varname(id)
    }
    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.read().unwrap().get_typeid(tn)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
