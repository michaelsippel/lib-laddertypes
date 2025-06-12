use crate::{heuristic::*, dict::*, parser::*, morphism::*};

#[test]
fn test_heuristic() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("A").expect("parse"),
            dst_type: dict.parse("A").expect("parse")
        }.estimated_cost(),
        0
    );

    assert_eq!(
        MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("<Digit 10> ~ Char ~ Ascii ~ native.UInt8").expect("parse"),
            dst_type: dict.parse("<Digit 16> ~ native.UInt8").expect("parse")
        }.estimated_cost(),
        40
    );
}
