use crate::{dict::*, term::*};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait UnparseLadderType {
    fn unparse(&self, t: &TypeTerm) -> String;
}

impl<T: TypeDict> UnparseLadderType for T {
    fn unparse(&self, t: &TypeTerm) -> String {
        match t {
            TypeTerm::Id(id) => {
                self.get_typename(*id).unwrap_or("?Fun?".into())
            }
            TypeTerm::Var(id) => {
                self.get_varname(*id).unwrap_or("?Var?".into())
            }
            TypeTerm::Num(n) => format!("{}", n),
            TypeTerm::Char(c) => match c {
                '\0' => "'\\0'".into(),
                '\n' => "'\\n'".into(),
                '\t' => "'\\t'".into(),
                '\'' => "'\\''".into(),
                c => format!("'{}'", c)
            }
            TypeTerm::Ladder(rungs) => {
                let mut s = String::new();
                let mut first = true;
                for r in rungs.iter() {
                    if !first {
                        s.push('~');
                    }
                    first = false;
                    s.push_str(&mut self.unparse(r));
                }
                s
            }
            TypeTerm::Spec(args) => {
                let mut s = String::new();
                s.push('<');
                let mut first = true;
                for r in args.iter() {
                    if !first {
                        s.push(' ');
                    }
                    first = false;
                    s.push_str(&mut self.unparse(r));
                }
                s.push('>');
                s
            }
            TypeTerm::Seq { seq_repr, items } => {
                todo!()
            }
            TypeTerm::Struct { struct_repr, members } => {
                todo!()
            }
            TypeTerm::Enum { enum_repr, variants } => {
                todo!()
            }
            TypeTerm::Univ(bound, t) => {
                todo!()
            }
            TypeTerm::Func(ts) => {
                todo!()
            }
            TypeTerm::Morph(s, t) => {
                todo!()
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
