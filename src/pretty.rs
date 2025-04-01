use {
    crate::{dict::TypeID, sugar::SugaredTypeTerm, SugaredStructMember, SugaredEnumVariant, TypeDict},
    tiny_ansi::TinyAnsi
};


impl SugaredStructMember {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}
impl SugaredEnumVariant {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}

impl SugaredTypeTerm {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        let indent_width = 4;
        match self {
            SugaredTypeTerm::TypeID(id) => {
                match id {
                    TypeID::Var(varid) => {
                        format!("{}", dict.get_typename(id).unwrap_or("??".bright_red())).bright_magenta()
                    },
                    TypeID::Fun(funid) => {
                        format!("{}", dict.get_typename(id).unwrap_or("??".bright_red())).blue().bold()
                    }
                }
            },

            SugaredTypeTerm::Num(n) => {
                format!("{}", n).green().bold()
            }

            SugaredTypeTerm::Char(c) => {
                match c {
                    '\0' => format!("'\\0'"),
                    '\n' => format!("'\\n'"),
                    _ => format!("'{}'", c)
                }
            }

            SugaredTypeTerm::Univ(t) => {
                format!("{} {} . {}",
                    "∀".yellow().bold(),
                    dict.get_varname(0).unwrap_or("??".into()).bright_blue(),
                    t.pretty(dict,indent)
                )
            }

            SugaredTypeTerm::Spec(args) => {
                let mut s = String::new();
                s.push_str(&"<".yellow());
                for i in 0..args.len() {
                    let arg = &args[i];
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str( &arg.pretty(dict,indent+1) );
                }
                s.push_str(&">".yellow());
                s
            }

            SugaredTypeTerm::Struct{ struct_repr, members } => {
                let mut s = String::new();
                s.push_str(&"{".yellow().bold());

                if let Some(struct_repr) = struct_repr {
                    s.push_str(&format!("{}{} ", "~".yellow(), struct_repr.pretty(dict, indent+1)));
                }

                for member in members {
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    s.push_str(&member.pretty(dict, indent + 1));
                    s.push_str(&";\n".bright_yellow());
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&"}".yellow().bold());
                s
            }

            SugaredTypeTerm::Enum{ enum_repr, variants } => {
                let mut s = String::new();
                s.push_str(&"(".yellow().bold());

                if let Some(enum_repr) = enum_repr {
                    s.push_str(&format!("{}{} ", "~".yellow(), enum_repr.pretty(dict, indent+1)));
                }


                for (i,variant) in variants.iter().enumerate() {
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    if i > 0 {
                        s.push_str(&"| ".yellow().bold());
                    }
                    s.push_str(&variant.pretty(dict, indent + 1));
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&")".yellow().bold());
                s
            }

            SugaredTypeTerm::Seq{ seq_repr, items } => {
                let mut s = String::new();
                s.push_str(&"[".yellow().bold());

                if let Some(seq_repr) = seq_repr {
                    s.push_str(&format!("{}{}", "~".yellow(), seq_repr.pretty(dict, indent+1)));
                }
                s.push(' ');

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str(&item.pretty(dict, indent+1));
                }
                s.push_str(&" ]".yellow().bold());
                s
            }

            SugaredTypeTerm::Morph(args) => {
                let mut s = String::new();
                for arg in args {
                    s.push_str(&"  ~~morph~~>  ".bright_yellow());
                    s.push_str(&arg.pretty(dict, indent));
                }
                s
            }

            SugaredTypeTerm::Func(args) => {
                let mut s = String::new();
                for i in 0..args.len() {
                    let arg = &args[i];
                    if i > 0{
                        s.push('\n');
                        for x in 0..(indent*indent_width) {
                            s.push(' ');
                        }
                        s.push_str(&"-->  ".bright_yellow());
                    } else {
//                        s.push_str("   ");
                    }
                    s.push_str(&arg.pretty(dict, indent));
                }
                s
            }

            SugaredTypeTerm::Ladder(rungs) => {
                let mut s = String::new();
                for i in 0..rungs.len() {
                    let rung = &rungs[i];
                    if i > 0{
                        s.push('\n');
                        for x in 0..(indent*indent_width) {
                            s.push(' ');
                        }
                        s.push_str(&"~ ".yellow());
                    }
                    s.push_str(&rung.pretty(dict, indent));
                }
                s
            }
        }
    }
}
