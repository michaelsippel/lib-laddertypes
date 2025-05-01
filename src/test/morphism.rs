use {
    crate::{dict::*, morphism_base::MorphismBase,
        morphism_path::ShortestPathProblem,
        morphism::{MorphismInstance, Morphism, MorphismType},
        parser::*, TypeTerm,
        DesugaredTypeTerm
    },
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

fn print_subst(m: &std::collections::HashMap<TypeID, TypeTerm>, dict: &mut impl TypeDict) {
    eprintln!("{{");

    for (k,v) in m.iter() {
        eprintln!("    {} --> {}",
            dict.get_typename(k).unwrap(),
            v.pretty(dict, 0)
        );
    }

    eprintln!("}}");
}

fn print_path(dict: &mut impl TypeDict, path: &Vec<MorphismInstance<DummyMorphism>>) {
    for n in path.iter() {
        eprintln!("
morph {}
--> {}
with
        ",
        n.get_type().src_type.pretty(dict, 0),
        n.get_type().dst_type.pretty(dict, 0),
        );
        print_subst(&n.get_subst(), dict)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq)]
struct DummyMorphism(MorphismType);
impl Morphism for DummyMorphism {
    fn get_type(&self) -> MorphismType {
        self.0.clone()
    }
}

fn morphism_test_setup() -> ( BimapTypeDict, MorphismBase<DummyMorphism> ) {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    dict.add_varname("Radix".into());
    dict.add_varname("SrcRadix".into());
    dict.add_varname("DstRadix".into());

    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict)
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict)
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict)
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix>~ℤ_2^64~machine.UInt64>").unwrap().sugar(&mut dict)
        })
    );

    (dict, base)
}

#[test]
fn test_morphism_path1() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse_desugared("<Digit 10> ~ Char").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("<Digit 10> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict),
    }).solve();

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType {
                        src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                    }),
                }
            ]
    ));
}


#[test]
fn test_morphism_path2() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
    }).solve();

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                        }),
                    })
                }
            ]
    ));
}


#[test]
fn test_morphism_path3() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse_desugared("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 10 LittleEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                        }),
                    })
                },

                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType {
                        src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                }
            ]
    ));
}



#[test]
fn test_morphism_path4() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse_desugared("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().sugar(&mut dict)
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 10 LittleEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                        }),
                    })
                },

                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType {
                        src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                },

                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 16 LittleEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict)
                        }),
                    })
                },
            ]
    ));
}




#[test]
fn test_morphism_path_posint() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().sugar(&mut dict),
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                        }),
                    })
                },

                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        src_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                },
                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                },
                MorphismInstance::Primitive {
                    σ: vec![
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        src_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                    }),
                },

                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 16 BigEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            src_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict)
                        }),
                    })
                },
            ]
        )
    );
/*
    assert_eq!(
        base.find_morphism_path(MorphismType {
            src_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
        }),
        Some(
            vec![
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().normalize(),
            ]
        )
    );
    */


/*
    assert_eq!(
        base.find_morphism_with_subtyping(
            &MorphismType {
                src_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
                dst_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
            }
        ),

        Some((
                DummyMorphism(MorphismType{
                    src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                    dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                }),

                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian>").unwrap(),

                vec![
                    (dict.get_typeid(&"Radix".into()).unwrap(),
                    dict.parse("10").unwrap())
                ].into_iter().collect::<std::collections::HashMap<TypeID, TypeTerm>>()
        ))
    );
    */
}

#[test]
fn morphism_test_seq_repr() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> native.UInt8>").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> native.UInt8>").unwrap().sugar(&mut dict)
        })
    );

    assert_eq!(
        base.get_morphism_instance(&MorphismType {
            src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> Char~Ascii~native.UInt8>").expect("parse").sugar(&mut dict),
            dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> Char~Ascii~native.UInt8>").expect("parse").sugar(&mut dict)
        }),
        Some(
            MorphismInstance::Primitive {
                ψ: dict.parse_desugared("<Seq Char~Ascii>").expect("").sugar(&mut dict),
                σ: HashMap::new(),
                morph: DummyMorphism(MorphismType{
                    src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> native.UInt8>").unwrap().sugar(&mut dict),
                    dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> native.UInt8>").unwrap().sugar(&mut dict)
                })
            }
        )
    );
}

/*
use std::collections::HashMap;

#[test]
fn test_morphism_path_listedit()
{
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new( vec![ dict.parse("List").expect("") ] );

    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("Char").unwrap(),
            dst_type: dict.parse("Char ~ EditTree").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char>").unwrap(),
            dst_type: dict.parse("<List Char>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List Char>").unwrap(),
            dst_type: dict.parse("<List Char~ReprTree>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List ReprTree>").unwrap(),
            dst_type: dict.parse("<List~Vec ReprTree>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
            dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
            dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
        })
    );


    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse("<Seq~List~Vec <Digit 10>~Char>").unwrap(),
        dst_type: dict.parse("<Seq~List <Digit 10>~Char> ~ EditTree").unwrap(),
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(vec![
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List~Vec Char>").unwrap(),
                    dst_type: dict.parse("<List Char>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List Char>").unwrap(),
                    dst_type: dict.parse("<List Char~ReprTree>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List ReprTree>").unwrap(),
                    dst_type: dict.parse("<List~Vec ReprTree>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>~Char>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
                    dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
        ])
    );
}
*/
