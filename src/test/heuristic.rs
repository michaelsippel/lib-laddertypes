use crate::{heuristic::*, dict::*, parser::*, morphism_graph::*};

#[test]
fn test_heuristic() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        estimated_morphism_cost(
            &MorphismType {
                bounds: Vec::new(),
                src_type: dict.parse("A").expect("parse"),
                dst_type: dict.parse("A").expect("parse")
            }),
        1
    );

    assert_eq!(
        estimated_morphism_cost(
            &MorphismType {
                bounds: Vec::new(),
                src_type: dict.parse("<Digit 10> ~ Char ~ Ascii ~ native.UInt8").expect("parse"),
                dst_type: dict.parse("<Digit 16> ~ native.UInt8").expect("parse")
            }),
        41
    );
}
