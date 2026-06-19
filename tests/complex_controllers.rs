use abstract_bits::{AbstractBits, abstract_bits};

// Inverted presence: the option is present when the controller is false.
#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
struct Inverted {
    flag: bool,
    #[abstract_bits(presence_from = !flag)]
    opt: Option<u16>,
    trailing: u8,
}

#[test]
fn inverted_presence_present() {
    let value = Inverted {
        flag: false,
        opt: Some(0x1234),
        trailing: 0x5A,
    };
    let bytes = value.to_abstract_bits().unwrap();
    assert_eq!(Inverted::from_abstract_bits(&bytes).unwrap(), value);
}

#[test]
fn inverted_presence_absent() {
    let value = Inverted {
        flag: true,
        opt: None,
        trailing: 0x5A,
    };
    let bytes = value.to_abstract_bits().unwrap();
    assert_eq!(Inverted::from_abstract_bits(&bytes).unwrap(), value);
}

// One controller driving several options. `present` is a bare same-struct field, so it
// is hidden and derived from the first option on write; both options are present or
// both absent.
#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
struct SharedPresence {
    present: bool,
    #[abstract_bits(presence_from = present)]
    a: Option<u8>,
    #[abstract_bits(presence_from = present)]
    b: Option<u16>,
    trailing: u8,
}

#[test]
fn shared_presence_present() {
    let value = SharedPresence {
        a: Some(0x11),
        b: Some(0x2222),
        trailing: 0x5A,
    };
    let bytes = value.to_abstract_bits().unwrap();
    assert_eq!(SharedPresence::from_abstract_bits(&bytes).unwrap(), value);
}

#[test]
fn shared_presence_absent() {
    let value = SharedPresence {
        a: None,
        b: None,
        trailing: 0x5A,
    };
    let bytes = value.to_abstract_bits().unwrap();
    assert_eq!(SharedPresence::from_abstract_bits(&bytes).unwrap(), value);
}
