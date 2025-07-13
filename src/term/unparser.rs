use crate::{dict::*, desugared_term::*};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait UnparseLadderType {
    fn unparse(&self, t: &DesugaredTypeTerm) -> String;
}

impl<T: TypeDict> UnparseLadderType for T {
    fn unparse(&self, t: &DesugaredTypeTerm) -> String {
        match t {
            DesugaredTypeTerm::TypeID(TypeID::Fun(id)) => {
                self.get_typename(*id).unwrap_or("?Fun?".into())
            },
            DesugaredTypeTerm::TypeID(TypeID::Var(id)) => {
                self.get_varname(*id).unwrap_or("?Var?".into())
            },
            DesugaredTypeTerm::Num(n) => format!("{}", n),
            DesugaredTypeTerm::Char(c) => match c {
                '\0' => "'\\0'".into(),
                '\n' => "'\\n'".into(),
                '\t' => "'\\t'".into(),
                '\'' => "'\\''".into(),
                c => format!("'{}'", c)
            },
            DesugaredTypeTerm::Ladder(rungs) => {
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
            DesugaredTypeTerm::App(args) => {
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
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
