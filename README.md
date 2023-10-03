# lib-laddertypes

Rust Implementation of Ladder-Types (parsing, unification, rewriting, etc) 
<hr/>

## Ladder Types

In order to implement complex datastructures and algorithms, usually
many layers of abstraction are built ontop of each other.
Consequently higher-level data types are encoded into lower-level data
types, forming a chain of embeddings from concept to `rock bottom' of
byte streams.  While a high-level type makes claims about the
semantics of objects of that type, high-level types are ambiguous in
regard to their concrete syntactical representation or memory
layout. However for compositions to be type-safe, compatibility of
concrete represenations must be ensured.

For example in the unix shell, many different tools & utilities
coexist.  Depending on the application domain, each of them will
potentially make use of different representational forms for the same
abstract concepts. E.g. for the concept 'natural number', many
representations do exist, e.g. with variation over radices,
endianness, digit encoding etc.

Intuitively, *ladder types* provide a way to distinguish between
multiple *concrete representations* of the same *abstract / conceptual
type*, by capturing the *represented-as* of layered data formats in
the structure of type-terms. Formally, we introduce a new type
constructor, called the *ladder type*, written `T1 ~ T2`, where `T1`
and `T2` are types. The type-term `T1 ~ T2` then expresses the
abstract type of `T1` being represented in terms of the concrete type
`T2`, which can be read by "`T1` represented as `T2`".


#### Example
The following type describes a colon-separated sequence of timepoints,
each represented as unix-timestamp written as decimal number in
big-endian, encoded as UTF-8 string.

```
<Seq TimePoint
     ~<TimeSince UnixEpoch>
     ~<Duration Seconds>
     ~ℕ
     ~<PosInt 10 BigEndian>
     ~<Seq <Digit 10>~Char>>
~<SepSeq Char ':'>
~<Seq Char>
~UTF-8
~<Seq Byte>
```

An object that fits the format described by this type could look like
this:

```
1696093021:1696093039:1528324679:1539892301:1638141920:1688010253
```

## How to use this crate

```rust
use laddertypes::*;

fn main() {
    let mut dict = TypeDict::new();

    let t1 = dict.parse("<A B~X C>").expect("couldnt parse typeterm");
    let t2 = dict.parse("<<A B~X> C>").expect("couldnt parse typeterm");

    assert_eq!( t1.clone().curry(), t2 );
    assert_eq!( t1, t2.clone().decurry() );
}
```


## License
[GPLv3](COPYING)

