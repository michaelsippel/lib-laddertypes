use crate::{dict::*, term::*};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeDict {
    pub fn unparse(&self, t: &TypeTerm) -> String {
        match t {
            TypeTerm::TypeID(id) => self.get_typename(id).unwrap(),
            TypeTerm::Num(n) => format!("{}", n),
            TypeTerm::Char(c) => match c {
                '\0' => "'\\0'".into(),
                '\n' => "'\\n'".into(),
                '\t' => "'\\t'".into(),
                '\'' => "'\\''".into(),
                c => format!("'{}'", c)
            },
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
            TypeTerm::App(args) => {
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
