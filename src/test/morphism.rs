use {
    crate::{dict::*, morphism::*, parser::*, unparser::*, TypeTerm}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

fn print_subst(m: &std::collections::HashMap<TypeID, TypeTerm>, dict: &mut impl TypeDict) {
    eprintln!("{{");

    for (k,v) in m.iter() {
        eprintln!("    {} --> {}",
            dict.get_typename(k).unwrap(),
            dict.unparse(v)
        );
    }

    eprintln!("}}");
}

fn print_path(dict: &mut impl TypeDict, path: &Vec<MorphismInstance<DummyMorphism>>) {
    for n in path.iter() {
        eprintln!("
ψ = {}
morph {}
--> {}
with
        ",
        n.halo.clone().sugar(dict).pretty(dict, 0),
        n.m.get_type().src_type.sugar(dict).pretty(dict, 0),
        n.m.get_type().dst_type.sugar(dict).pretty(dict, 0),
        );
        print_subst(&n.σ, dict)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq)]
struct DummyMorphism(MorphismType);

impl Morphism for DummyMorphism {
    fn get_type(&self) -> MorphismType {
        self.0.clone().normalize()
    }

    fn map_morphism(&self, seq_type: TypeTerm) -> Option<DummyMorphism> {
        Some(DummyMorphism(MorphismType {
            src_type: TypeTerm::App(vec![
                seq_type.clone(),
                self.0.src_type.clone()
            ]),

            dst_type: TypeTerm::App(vec![
                seq_type.clone(),
                self.0.dst_type.clone()
            ])
        }))
    }
}

fn morphism_test_setup() -> ( BimapTypeDict, MorphismBase<DummyMorphism> ) {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new( vec![ dict.parse("Seq").expect("") ] );

    dict.add_varname("Radix".into());
    dict.add_varname("SrcRadix".into());
    dict.add_varname("DstRadix".into());

    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ Char").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );

    (dict, base)
}

#[test]
fn test_morphism_path1() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("<Digit 10> ~ Char").unwrap(),
        dst_type: dict.parse("<Digit 10> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
    });

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
                        dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                    }),
                }
            ]
    ));
}


#[test]
fn test_morphism_path2() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
        dst_type: dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
    });

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 10 BigEndian>").expect(""),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                        dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                }
            ]
    ));
}


#[test]
fn test_morphism_path3() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
        dst_type: dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
    });

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                        dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },

                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                }
            ]
    ));
}



#[test]
fn test_morphism_path4() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
        dst_type: dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
    });

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                        dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },

                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },

                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 16 LittleEndian>").expect(""),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),                        
                        dst_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap()
                    }),
                },
                
            ]
    ));
}




#[test]
fn test_morphism_path_posint() {
    let (mut dict, mut base) = morphism_test_setup();

    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
        dst_type: dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap(),
    });

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 10 BigEndian>").unwrap(),
                    m: DummyMorphism(MorphismType {
                        src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                        dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType{
                        src_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"SrcRadix".into()).unwrap(), TypeTerm::Num(10)),
                        (dict.get_typeid(&"DstRadix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType{
                        src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                    }),
                },
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    halo: TypeTerm::unit(),
                    m: DummyMorphism(MorphismType{
                        src_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                    }),
                },
                MorphismInstance {
                    σ: vec![
                        (dict.get_typeid(&"Radix".into()).unwrap(), TypeTerm::Num(16))
                    ].into_iter().collect(),
                    halo: dict.parse("ℕ ~ <PosInt 16 BigEndian>").unwrap(),
                    m: DummyMorphism(MorphismType{
                        src_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
                        dst_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap()
                    })
                }
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


    let path = base.find_morphism_path(MorphismType {
        src_type: dict.parse("<Seq~List~Vec <Digit 10>~Char>").unwrap(),
        dst_type: dict.parse("<Seq~List <Digit 10>~Char> ~ EditTree").unwrap(),
    });

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
