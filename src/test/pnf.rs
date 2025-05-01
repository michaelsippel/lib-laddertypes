use crate::{dict::BimapTypeDict, parser::*};

#[test]
fn test_normalize_id() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        dict.parse("A~B~C").expect("parse error"),
        dict.parse("A~B~C").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A B>~C").expect("parse error"),
        dict.parse("<A B>~C").expect("parse error").normalize(),
    );
}

#[test]
fn test_normalize_spec() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        dict.parse("<A B~C>").expect("parse error"),
        dict.parse("<A B>~<A C>").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A~Y B>").expect("parse error"),
        dict.parse("<A~Y B>~<Y B>").expect("parse error").normalize(),
    );

    assert_eq!(
        dict.parse("<A B~C D~E>").expect("parse error"),
        dict.parse("<A B D>~<A C D>~<A C E>").expect("parse errror").normalize(),
    );

    assert_eq!(
        dict.parse("<A~X B~C D~E>").expect("parse error"),
        dict.parse("<A~X B D>~<A~X B~C E>~<X C E>").expect("parse errror").normalize(),
    );
}

#[test]
fn test_normalize_seq() {
    let mut dict = BimapTypeDict::new();
    assert_eq!(
        dict.parse("<Seq Char~Ascii>").expect("parse error"),
        dict.parse("<Seq Char>~<Seq Ascii>").expect("parse errror").normalize(),
    );

    eprintln!("---------------");
    assert_eq!(
        dict.parse("<Seq <Digit 10>~Char>").expect("parse error"),
        dict.parse("<Seq <Digit 10>>~<Seq Char>").expect("parse errror").normalize(),
    );
    eprintln!("---------------");
    assert_eq!(
        dict.parse("<Seq~<ValueDelim '\\0'> Char~Ascii~native.UInt8>").expect("parse error"),
        dict.parse("<Seq Char> ~ <<ValueDelim '\\0'> Char> ~ <<ValueDelim '\\0'> Ascii~native.UInt8>").expect("parse error").normalize(),
    );

    eprintln!("---------------");
    assert_eq!(
        dict.parse("<Seq~<ValueDelim '\\0'> Char~Ascii~native.UInt8>").expect("parse error"),
        dict.parse("<Seq Char~Ascii> ~ <<ValueDelim '\\0'> Char~Ascii> ~ <<ValueDelim '\\0'> native.UInt8>").expect("parse error").normalize(),
    );
}

#[test]
fn test_normalize_complex_spec() {
    let mut dict = BimapTypeDict::new();
    assert_eq!(
        dict.parse("<A~Y <B C~D~E> F H H>").expect("parse error"),
        dict.parse("<A~Y <B C> F H H>
                   ~<A~Y <B D> F H H>
                   ~<Y <B E> F H H>").expect("parse errror")
               .normalize(),
    );
}

#[test]
fn test_normalize_struct() {
    let mut dict = BimapTypeDict::new();
    assert_eq!(
        dict.parse("< Struct~Aligned
                <  a   TimePoint~<TimeSince UnixEpoch>~Seconds~native.UInt64  >
                <  b   Angle ~ Degrees ~ ℝ ~ native.Float32 >
            >
            ").expect("parse error"),
        dict.parse("
            < Struct <a TimePoint> <b Angle> >
        ~   < Struct  <a <TimeSince UnixEpoch>~Seconds> <b Angle~Degrees~ℝ> >
        ~   < Struct~Aligned  <a native.UInt64> <b native.Float32> >
        ").expect("parse errror")

            .normalize(),
    );
}

#[test]
fn test_normalize_enum() {
    let mut dict = BimapTypeDict::new();
    assert_eq!(
        dict.parse("< Enum
                <  a   TimePoint~<TimeSince UnixEpoch>~Seconds~native.UInt64  >
                <  b   Angle ~ Degrees ~ ℝ ~ native.Float32 >
            >
            ").expect("parse error"),
        dict.parse("
            < Enum <a TimePoint> <b Angle> >
        ~   < Enum  <a <TimeSince UnixEpoch>~Seconds> <b Angle~Degrees~ℝ> >
        ~   < Enum  <a native.UInt64> <b native.Float32> >
        ").expect("parse errror")

            .normalize(),
    );
}
