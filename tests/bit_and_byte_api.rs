//! Tests for the bits and bytes APIs.
use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits]
#[derive(Debug, PartialEq)]
struct SubByte {
    head: u8,
    tail: u3, // total = 11 bits, not a multiple of 8
}

#[test]
fn bits_buffer_has_exact_length() {
    let bits = SubByte { head: 0xAB, tail: 5 }.to_abstract_bits().unwrap();
    assert_eq!(bits.len(), 11);
    assert_eq!(bits.len(), SubByte::MAX_BITS);
}

#[test]
fn roundtrips_through_bits() {
    let value = SubByte { head: 0xAB, tail: 5 };
    let bits = value.to_abstract_bits().unwrap();
    assert_eq!(SubByte::from_abstract_bits(&bits).unwrap(), value);
}

#[test]
fn roundtrips_through_bytes_with_tail_padding() {
    let value = SubByte { head: 0xAB, tail: 5 };
    let bytes = value.to_abstract_bytes().unwrap();
    assert_eq!(bytes.len(), 2); // 11 bits padded up to 2 whole bytes
    assert_eq!(SubByte::from_abstract_bytes(&bytes).unwrap(), value);
}

#[test]
fn bytes_are_the_bit_buffer_padded_to_a_byte() {
    let value = SubByte { head: 0xAB, tail: 5 };
    let bits = value.to_abstract_bits().unwrap();
    assert_eq!(value.to_abstract_bytes().unwrap(), bits.into_vec());
}
