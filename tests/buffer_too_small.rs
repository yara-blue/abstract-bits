use abstract_bits::{AbstractBits, BitWriter};

// Writing past the buffer must report the bit deficit, not underflow computing it.

#[test]
fn deficit_into_empty_buffer() {
    let mut buf = [0u8; 1];
    let mut writer = BitWriter::from(&mut buf[..]);

    let err = 0xABCDu16.write_abstract_bits(&mut writer).unwrap_err();
    let cause = std::error::Error::source(&err).unwrap();
    assert!(cause.to_string().contains("8 bits extra"));
}

#[test]
fn deficit_accounts_for_position() {
    let mut buf = [0u8; 2];
    let mut writer = BitWriter::from(&mut buf[..]);

    0xABu8.write_abstract_bits(&mut writer).unwrap();

    let err = 0xCDEFu16.write_abstract_bits(&mut writer).unwrap_err();
    let cause = std::error::Error::source(&err).unwrap();
    assert!(cause.to_string().contains("8 bits extra"));
}
