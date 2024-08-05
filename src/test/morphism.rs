use {
    crate::{dict::*, morphism::*, parser::*, TypeTerm}
};

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

#[test]
fn test_morphism_path() {
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

    assert_eq!(
        base.find_morphism_path(MorphismType {
            src_type: dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
        }),
        Some(
            vec![
                dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().normalize(),
            ]
        )
    );

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
}
