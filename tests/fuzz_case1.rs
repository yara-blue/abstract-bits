use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
struct Frame {
    header: u4,
    has_source: bool,
    data_len: u5,
    ty: Type,
    #[abstract_bits(presence_from = has_source)]
    source: Option<u16>,
    #[abstract_bits(length_from = data_len)]
    data: Vec<Message>,
}

/// This is: 4+3+1+10 = 18 bits long
#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
struct Message {
    header: u4,
    reserved: u3,
    is_important: bool,
    bits: [bool; 10],
}

#[abstract_bits(bits = 2)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Type {
    #[default]
    System = 0,
    Personal = 1,
    Group = 2,
}

#[test]
fn test_test_case() {
    color_eyre::install().unwrap();

    let serialized = test_input().to_abstract_bytes().unwrap();
    let deserialized = Frame::from_abstract_bytes(&serialized).unwrap();
    assert_eq!(test_input(), deserialized)
}

fn test_input() -> Frame {
    Frame {
        header: 9,
        ty: Type::System,
        source: Some(63479),
        data: vec![
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 9,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 1,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [
                    true, true, true, false, false, false, false, false, false, true,
                ],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 2,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 15,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, true, true, true],
            },
            Message {
                header: 7,
                is_important: true,
                bits: [true, true, true, true, true, true, true, false, true, true],
            },
        ],
    }
}
