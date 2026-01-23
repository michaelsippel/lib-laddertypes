/*
   lib-laddertypes
   Copyright (C) 2023-2026  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

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
            TypeTerm::Seq { seq_repr, item } => {
                let mut s = String::new();
                s.push('[');

                if let Some(sr) = seq_repr {
                    s.push_str("~");
                    s.push_str(&mut self.unparse(sr));
                    s.push_str(" ");
                }
                

                s.push_str(&self.unparse(&item));
                s.push(']');
                s
            }
            TypeTerm::Struct { struct_repr, members } => {
                let mut s = String::new();
                s.push('{');

                if let Some(sr) = struct_repr {
                    s.push_str("~");
                    s.push_str(&mut self.unparse(sr));
                    s.push_str(" ");
                }
                
                for m in members.iter() {
                    s.push_str(&m.symbol);
                    s.push_str(": ");
                    s.push_str(&self.unparse(&m.ty));
                    s.push_str("; ");
                }

                s.push('}');
                s
            }
            TypeTerm::Enum { enum_repr, variants } => {
                let mut s = String::new();
                s.push('{');

                if let Some(sr) = enum_repr {
                    s.push_str("~");
                    s.push_str(&mut self.unparse(sr));
                    s.push_str(" ");
                }
                
                for m in variants.iter() {

                    s.push_str("| ");
                    s.push_str(&m.symbol);
                    s.push_str(": ");
                    s.push_str(&self.unparse(&m.ty));
                }

                s.push('}');
                s
            }
            TypeTerm::Univ{ Γ, bounds, τ } => {
                let mut s = String::new();

                for entry in Γ.iter() {
                    s.push_str("∀");
                    s.push_str(&entry.symbol);
                    match &entry.kind {
                        crate::TypeKind::Type => {},
                        crate::TypeKind::Arrow(_type_kind, _type_kind1) => {
                            todo!();
                        },
                        crate::TypeKind::Value(type_term) => {
                            s.push_str(":");
                            s.push_str(&self.unparse(&type_term));
                        }
                    }
                    s.push_str(" ");
                }

                for bound in bounds {
                    s.push_str("(");
                    match bound {
                        ConstraintPair::ValueOf(type_term, type_term1) => {
                            s.push_str(&self.unparse(type_term));
                            s.push_str(":");
                            s.push_str(&self.unparse(type_term1));
                        },
                        ConstraintPair::Subtype(type_term, type_term1) => {
                            s.push_str(&self.unparse(type_term));
                            s.push_str(" <= ");
                            s.push_str(&self.unparse(type_term1));
                        },
                        ConstraintPair::Trait(type_term, type_term1) => {
                            s.push_str(&self.unparse(type_term));
                            s.push_str(" >< ");
                            s.push_str(&self.unparse(type_term1));
                        },
                        ConstraintPair::Parallel(type_term, type_term1) => {
                            s.push_str(&self.unparse(type_term));
                            s.push_str(" || ");
                            s.push_str(&self.unparse(type_term1));
                        },
                    }
                    s.push_str(") ");
                }

                s.push_str(&self.unparse(&τ));

                s
            }
            TypeTerm::Func(ts) => {
                let mut s = String::new();

                let mut first = true;
                for x in ts.iter() {
                    if !first {
                        s.push_str("-->");
                    } else {
                        first = false;
                    }
                    s.push_str(&self.unparse(x));
                }

                s

            }
            TypeTerm::Morph(s, t) => {
                let mut st = String::new();
                st.push_str(&self.unparse(s));
                st.push_str("-morph->");
                st.push_str(&self.unparse(t));
                st
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
