use abstract_bits::{AbstractBits, BitWriter, BufferTooSmall, ToBytesError};

// Writing past the buffer must report the bit deficit.

#[test]
fn deficit_into_empty_buffer() {
    let mut buf = [0u8; 1];
    let mut writer = BitWriter::from(&mut buf[..]);

    let err = 0xABCDu16.write_abstract_bits(&mut writer).unwrap_err();
    assert_eq!(
        err,
        ToBytesError::BufferTooSmall {
            ty: "u16",
            cause: BufferTooSmall {
                n_bits: 16,
                bits_needed: 8,
            },
        }
    );
}

#[test]
fn deficit_accounts_for_position() {
    let mut buf = [0u8; 2];
    let mut writer = BitWriter::from(&mut buf[..]);

    0xABu8.write_abstract_bits(&mut writer).unwrap();

    let err = 0xCDEFu16.write_abstract_bits(&mut writer).unwrap_err();
    assert_eq!(
        err,
        ToBytesError::BufferTooSmall {
            ty: "u16",
            cause: BufferTooSmall {
                n_bits: 16,
                bits_needed: 8,
            },
        }
    );
}
