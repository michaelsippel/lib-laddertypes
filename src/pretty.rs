use {
    crate::{TypeDict, dict::TypeID},
    crate::sugar::SugaredTypeTerm,
    tiny_ansi::TinyAnsi
};

impl SugaredTypeTerm {
    pub fn pretty(&self, dict: &TypeDict, indent: u64) -> String {
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

            SugaredTypeTerm::Struct(args) => {
                let mut s = String::new();
                s.push_str(&"{".yellow().bold());
                for arg in args {
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    s.push_str(&arg.pretty(dict, indent + 1));
                    s.push_str(&";\n".bright_yellow());
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&"}".yellow().bold());
                s
            }

            SugaredTypeTerm::Enum(args) => {
                let mut s = String::new();
                s.push_str(&"(".yellow().bold());
                for i in 0..args.len() {
                    let arg = &args[i];
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    if i > 0 {
                        s.push_str(&"| ".yellow().bold());
                    }
                    s.push_str(&arg.pretty(dict, indent + 1));
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&")".yellow().bold());
                s
            }

            SugaredTypeTerm::Seq(args) => {
                let mut s = String::new();
                s.push_str(&"[ ".yellow().bold());
                for i in 0..args.len() {
                    let arg = &args[i];
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str(&arg.pretty(dict, indent+1));
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
