use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct SignedBitfields {
    small: i4,
    across_bytes: i12,
    reserved: u4,
    tiny: i2,
    filler: u2,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct NativeSigned {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct SignedContainers {
    len: u3,
    present: bool,
    reserved: u4,
    #[abstract_bits(presence_from = present)]
    offset: Option<i16>,
    #[abstract_bits(length_from = len)]
    deltas: Vec<i8>,
    #[abstract_bits(rest(max_bits = 32))]
    rest: Vec<i16>,
}

#[test]
fn sign_extension_roundtrip() {
    let s = SignedBitfields {
        small: -1,
        across_bytes: -1025,
        tiny: -2,
        filler: 0b11,
    };

    let bytes = s.to_abstract_bytes().unwrap();
    let parsed = SignedBitfields::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}

#[test]
fn twos_complement_bit_pattern() {
    let s = SignedBitfields {
        small: -3,
        across_bytes: -1,
        tiny: 1,
        filler: 0,
    };

    let bytes = s.to_abstract_bytes().unwrap();
    assert_eq!(
        bytes,
        vec![
            // across_bytes (low 4 bits) + small
            0b1111_1101,
            // across_bytes (high 8 bits)
            0b11111111,
            // filler + tiny + reserved
            0b00_01_0000,
        ]
    );
}

#[test]
fn native_signed_roundtrip() {
    let s = NativeSigned {
        a: i8::MIN,
        b: -2,
        c: i32::MAX,
        d: i64::MIN,
    };

    let bytes = s.to_abstract_bytes().unwrap();
    assert_eq!(bytes.len(), (8 + 16 + 32 + 64) / 8);
    assert_eq!(&bytes[..3], &[0x80, 0xfe, 0xff]);

    let parsed = NativeSigned::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}

#[test]
fn signed_containers_roundtrip() {
    let s = SignedContainers {
        offset: Some(-2000),
        deltas: vec![-1, 0, 127, -128],
        rest: vec![-1, i16::MIN],
    };

    let bytes = s.to_abstract_bytes().unwrap();
    let parsed = SignedContainers::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}
