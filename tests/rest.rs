//! The `rest` field mode: read a Vec until the reader is exhausted
use abstract_bits::{AbstractBits, abstract_bits};
use hex_literal::hex;

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct Frame {
    field1: u8,
    field2: u16,
    #[abstract_bits(rest, max_bytes = 16)]
    rest: Vec<u8>,
}

#[test]
fn rest_captures_trailing_bytes() {
    let bytes = hex!("aa ccbb 01 02 03").to_vec();
    let frame = Frame::from_abstract_bits(&bytes).unwrap();

    assert_eq!(
        frame,
        Frame {
            field1: 0xAA,
            field2: 0xBBCC,
            rest: vec![0x01, 0x02, 0x03],
        }
    );
    assert_eq!(frame.to_abstract_bits().unwrap(), bytes);
}

#[test]
fn rest_can_be_empty() {
    let frame = Frame {
        field1: 0x01,
        field2: 0x0002,
        rest: vec![],
    };
    let bytes = frame.to_abstract_bits().unwrap();

    assert_eq!(bytes, hex!("01 0200").to_vec());
    assert_eq!(Frame::from_abstract_bits(&bytes).unwrap(), frame);
}
