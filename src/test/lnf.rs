use crate::dict::TypeDict;

#[test]
fn test_flat() {
    let mut dict = TypeDict::new();

    assert!( dict.parse("A").expect("parse error").is_flat() );
    assert!( dict.parse("10").expect("parse error").is_flat() );
    assert!( dict.parse("'x'").expect("parse error").is_flat() );
    assert!( dict.parse("<A B 23>").expect("parse error").is_flat() );
    assert!( dict.parse("<A <B C 'x' D>>").expect("parse error").is_flat() );

    assert!( ! dict.parse("A~B").expect("parse error").is_flat() );
    assert!( ! dict.parse("<A B~C>").expect("parse error").is_flat() );
    assert!( ! dict.parse("<A <B C~X D>>").expect("parse error").is_flat() );
}

#[test]
fn test_normalize() {
    let mut dict = TypeDict::new();

    assert_eq!(
        dict.parse("A~B~C").expect("parse error").normalize(),
        dict.parse("A~B~C").expect("parse errror"),
    );

    assert_eq!(
        dict.parse("<A B>~C").expect("parse error").normalize(),
        dict.parse("<A B>~C").expect("parse errror"),
    );

    assert_eq!(
        dict.parse("<A B~C>").expect("parse error").normalize(),
        dict.parse("<A B>~<A C>").expect("parse errror"),
    );

    assert_eq!(
        dict.parse("<A B~C D~E>").expect("parse error").normalize(),
        dict.parse("<A B D>~<A C D>~<A C E>").expect("parse errror"),
    );

    assert_eq!(
        dict.parse("<Seq <Digit 10>~Char>").expect("parse error").normalize(),
        dict.parse("<Seq <Digit 10>>~<Seq Char>").expect("parse errror"),
    );


    assert_eq!(
        dict.parse("<A <B C~D~E> F~G H H>").expect("parse error").normalize(),
        dict.parse("<A <B C> F H H>
                   ~<A <B D> F H H>
                   ~<A <B E> F H H>
                   ~<A <B E> G H H>").expect("parse errror"),
    );

}

