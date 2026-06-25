use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits]
struct Register {
    device: u4,
    reserved: u1,
    on: bool,
    count: u2,
}

#[derive(Debug, PartialEq)]
#[abstract_bits]
struct UnalignedPack {
    field_a: u27,
    field_b: u7,
    field_c: u37,
    field_d: u9,
}

// #[abstract_bits]
// struct NormalStruct {
//     list: [bool; 5],
// }

// #[abstract_bits]
// struct UnitStruct([bool; 5]);

#[test]
fn main() {
    assert_eq!(Register::MIN_BITS, Register::MAX_BITS);
    assert_eq!(Register::MIN_BITS, 8);

    // assert_eq!(UnitStruct::MIN_BITS, UnitStruct::MAX_BITS);
    // assert_eq!(UnitStruct::MIN_BITS, 5);

    // assert_eq!(NormalStruct::MIN_BITS, NormalStruct::MAX_BITS);
    // assert_eq!(NormalStruct::MIN_BITS, 5);
}

#[test]
fn arbitrary_int_packing() {
    let s = UnalignedPack {
        field_a: 0b101_11111101_11111111_01111111,
        field_b: 0b10_10011,
        field_c: 0b1111011_11011111_11111111_01111111_111011,
        field_d: 0b10001000_1,
    };

    let bytes = s.to_abstract_bytes().unwrap();
    assert_eq!(
        bytes,
        vec![
            // field_a
            0b01111111,
            0b11111111,
            0b11111101,
            // field_a + field_b
            0b10011_101,
            // field_b + field_c
            0b111011_10,
            // field_c
            0b01111111,
            0b11111111,
            0b11011111,
            // field_c + field_d
            0b1_1111011,
            0b10001000,
        ]
    );

    let parsed = UnalignedPack::from_abstract_bytes(&bytes).unwrap();
    assert_eq!(parsed, s);
}
