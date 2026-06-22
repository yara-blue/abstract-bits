use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits(bits = 8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum Mode {
    A = 0,
    B = 1,
}

#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
struct Frame {
    mode: Mode,
    #[abstract_bits(presence_from = mode == Mode::B)]
    only_when_b: Option<u16>,
    #[abstract_bits(presence_from = mode != Mode::B)]
    only_when_not_b: Option<u8>,
    trailing: u8,
}

#[test]
fn enum_variant_gates_presence() {
    for frame in [
        Frame {
            mode: Mode::A,
            only_when_b: None,
            only_when_not_b: Some(0x42),
            trailing: 12,
        },
        Frame {
            mode: Mode::B,
            only_when_b: Some(0xABCD),
            only_when_not_b: None,
            trailing: 34,
        },
    ] {
        let bytes = frame.to_abstract_bits().unwrap();
        assert_eq!(Frame::from_abstract_bits(&bytes).unwrap(), frame);
    }
}
