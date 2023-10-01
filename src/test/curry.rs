use {
    crate::{term::*, dict::*, parser::*},
    std::str::FromStr
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_curry() {
    assert_eq!(
        TypeTerm::from_str("<A B C>").unwrap().curry(),
        TypeTerm::from_str("<<A B> C>").unwrap()
    );
    assert_eq!(
        TypeTerm::from_str("<A B C D>").unwrap().curry(),
        TypeTerm::from_str("<<<A B> C> D>").unwrap()
    );
    assert_eq!(
        TypeTerm::from_str("<A B C D E F G H I J K>").unwrap().curry(),
        TypeTerm::from_str("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap()
    );

    assert_eq!(
        TypeTerm::from_str("<A~X B C>").unwrap().curry(),
        TypeTerm::from_str("<<A~X B> C>").unwrap()
    );
    assert_eq!(
        TypeTerm::from_str("<A B C~Y~Z> ~ K").unwrap().curry(),
        TypeTerm::from_str("< <A B> C~Y~Z > ~ K").unwrap()
    );
}

#[test]
fn test_decurry() {
    assert_eq!(
        TypeTerm::from_str("<<A B> C>").unwrap().decurry(),
        TypeTerm::from_str("<A B C>").unwrap()
    );
    assert_eq!(
        TypeTerm::from_str("<<<A B> C> D>").unwrap().decurry(),
        TypeTerm::from_str("<A B C D>").unwrap(),
    );
    assert_eq!(
        TypeTerm::from_str("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap().decurry(),
        TypeTerm::from_str("<A B C D E F G H I J K>").unwrap()
    );
    
    assert_eq!(
        TypeTerm::from_str("<<A~X B> C>").unwrap().decurry(),
        TypeTerm::from_str("<A~X B C>").unwrap()
    );
    assert_eq!(
        TypeTerm::from_str("<<A~X B> C~Y>~K").unwrap().decurry(),
        TypeTerm::from_str("<A~X B C~Y> ~K").unwrap()
    );
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
