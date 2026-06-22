#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use abstract_bits::{abstract_bits, AbstractBits};

#[abstract_bits]
#[derive(Debug, Arbitrary, PartialEq, Eq)]
struct Frame {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=15))]
    header: u4,
    has_source: bool,
    data_len: u5,
    ty: Type,
    #[abstract_bits(presence_from = has_source)]
    source: Option<u16>,
    #[arbitrary(with = arbitrary_vec)]
    #[abstract_bits(length_from = data_len)]
    data: Vec<Message>,
}

fn arbitrary_vec(u: &mut Unstructured) -> arbitrary::Result<Vec<Message>> {
    let len = u.int_in_range(0..=31)?;
    (0..len).into_iter().map(|_| Message::arbitrary(u)).collect()
}

/// This is: 4+3+1+10 = 18 bits long
#[abstract_bits]
#[derive(Debug, Arbitrary, PartialEq, Eq)]
struct Message {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=15))]
    header: u4,
    reserved: u3,
    is_important: bool,
    bits: [bool; 10],
}

#[abstract_bits(bits = 2)]
#[derive(Default, Debug, Clone, Copy, Arbitrary, PartialEq, Eq)]
#[repr(u8)]
enum Type {
    #[default]
    System = 0,
    Personal = 1,
    Group = 2,
}

fuzz_target!(|frame: Frame| {
    let serialized = frame.to_abstract_bits().unwrap();
    let deserialized = Frame::from_abstract_bits(&serialized).unwrap();
    assert_eq!(frame, deserialized)
});
