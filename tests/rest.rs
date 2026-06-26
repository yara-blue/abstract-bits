//! The `rest` field mode: read a Vec until the reader is exhausted
use abstract_bits::{AbstractBits, abstract_bits};
use hex_literal::hex;

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct Frame {
    field1: u8,
    field2: u16,
    #[abstract_bits(rest, max_bits = 128)]
    rest: Vec<u8>,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct LargerFrame {
    field1: u8,
    field2: u16,
    #[abstract_bits(rest, max_bits = 128)]
    rest: Vec<u16>,
}

#[test]
fn rest_captures_trailing_bytes() {
    let bytes = hex!("aa ccbb 01 02 03").to_vec();
    let frame = Frame::from_abstract_bytes(&bytes).unwrap();

    assert_eq!(
        frame,
        Frame {
            field1: 0xAA,
            field2: 0xBBCC,
            rest: vec![0x01, 0x02, 0x03],
        }
    );
    assert_eq!(frame.to_abstract_bytes().unwrap(), bytes);
}

#[test]
fn rest_can_be_empty() {
    let frame = Frame {
        field1: 0x01,
        field2: 0x0002,
        rest: vec![],
    };
    let bytes = frame.to_abstract_bytes().unwrap();

    assert_eq!(bytes, hex!("01 0200").to_vec());
    assert_eq!(Frame::from_abstract_bytes(&bytes).unwrap(), frame);
}

#[test]
fn rest_max_length() {
    let bytes = hex!(
        "
        00
        11 22
        33 44 55 66 77 88 99 aa bb cc dd ee ff 01 02 03
        04 05 06 07 08 09
    "
    )
    .to_vec();

    let frame = Frame::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(
        frame,
        Frame {
            field1: 0x00,
            field2: 0x2211,
            rest: hex!("33 44 55 66 77 88 99 aa bb cc dd ee ff 01 02 03").to_vec(),
        }
    );
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct UnalignedFrame {
    header: u8,
    #[abstract_bits(rest, max_bits = 128)]
    rest: Vec<u7>,
}

#[test]
fn rest_unaligned_prefix_roundtrips() {
    let frame = UnalignedFrame {
        header: 0xAA,
        rest: vec![1, 2, 3, 4, 5],
    };
    let bits = frame.to_abstract_bits().unwrap();
    assert_eq!(UnalignedFrame::from_abstract_bits(&bits).unwrap(), frame);
}

#[abstract_bits]
#[derive(Debug, PartialEq, Clone)]
struct Triplet {
    a: u4,
    b: bool,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct StructRestFrame {
    header: u8,
    #[abstract_bits(rest, max_bits = 128)]
    rest: Vec<Triplet>,
}

#[test]
fn rest_unaligned_struct_roundtrips() {
    let frame = StructRestFrame {
        header: 0xAA,
        rest: vec![
            Triplet { a: 1, b: true },
            Triplet { a: 2, b: false },
            Triplet { a: 3, b: true },
        ],
    };
    let bits = frame.to_abstract_bits().unwrap();
    assert_eq!(StructRestFrame::from_abstract_bits(&bits).unwrap(), frame);
}

#[test]
fn rest_max_length_unaligned() {
    let bytes = hex!(
        "
        00
        11 22
        33 44 55 66 77 88 99 aa bb cc dd ee ff 01 02 03
        04
    "
    )
    .to_vec();

    let frame = LargerFrame::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(
        frame,
        LargerFrame {
            field1: 0x00,
            field2: 0x2211,
            rest: vec![
                0x4433, 0x6655, 0x8877, 0xaa99, 0xccbb, 0xeedd, 0x01ff, 0x0302,
            ],
        }
    );

    assert_eq!(frame.to_abstract_bytes().unwrap(), &bytes[0..bytes.len() - 1]);
}
