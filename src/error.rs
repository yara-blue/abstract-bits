use crate::{BufferTooSmall, UnexpectedEndOfBits};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReadErrorCause {
    #[error("Got invalid discriminant {got} while deserializing enum {ty}")]
    InvalidDiscriminant { ty: &'static str, got: usize },
    #[error("Could not deserialize primitive while deserializing {ty}")]
    NotEnoughInput {
        ty: &'static str,
        #[source]
        cause: UnexpectedEndOfBits,
    },
    #[error("Read error in manual AbstractBits implementation for {ty}")]
    Custom {
        ty: &'static str,
        #[source]
        cause: UnexpectedEndOfBits,
    },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FromBytesError {
    #[error(
        "Could not skip over specified bit padding \
        in {struct_name} while serializing"
    )]
    SkipPadding {
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error("Could not read {field_name} in struct {struct_name}")]
    ReadField {
        field_name: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error("Could not read Option {field_name} in struct {struct_name}")]
    ReadOption {
        field_name: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error(
        "Could not read field controlling option: {controlled_option_field} \
        in struct {struct_name}"
    )]
    ReadOptionController {
        controlled_option_field: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error("Could not read length for list {field_name} in struct {struct_name}")]
    ReadListLength {
        field_name: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error(
        "Could not read {list_len} items into list {field_name} 
        in struct {struct_name}"
    )]
    ReadList {
        list_len: usize,
        field_name: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error(
        "Could not read {array_len} items into array {field_name} 
        in struct {struct_name}"
    )]
    ReadArray {
        array_len: usize,
        field_name: &'static str,
        struct_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error("Could not read enum {enum_name}")]
    ReadEnum {
        enum_name: &'static str,
        #[source]
        cause: ReadErrorCause,
    },
    #[error(transparent)]
    ReadPrimitive(ReadErrorCause),
}

impl FromBytesError {
    pub fn skip_padding(self, struct_name: &'static str) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::SkipPadding { struct_name, cause }
        } else {
            self
        }
    }
    pub fn read_field(self, struct_name: &'static str, field_name: &'static str) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadField {
                field_name,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
    pub fn read_option(
        self,
        struct_name: &'static str,
        field_name: &'static str,
    ) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadOption {
                field_name,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
    pub fn read_option_controller(
        self,
        struct_name: &'static str,
        controlled_option_field: &'static str,
    ) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadOptionController {
                controlled_option_field,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
    pub fn read_list_length(
        self,
        struct_name: &'static str,
        field_name: &'static str,
    ) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadListLength {
                field_name,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
    pub fn read_list(
        self,
        struct_name: &'static str,
        field_name: &'static str,
        list_len: usize,
    ) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadList {
                list_len,
                field_name,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
    pub fn read_array(
        self,
        struct_name: &'static str,
        field_name: &'static str,
        array_len: usize,
    ) -> Self {
        if let Self::ReadPrimitive(cause) = self {
            Self::ReadArray {
                array_len,
                field_name,
                struct_name,
                cause,
            }
        } else {
            self
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ToBytesError {
    #[error("List too long to fit. Max length {max}, got: {got}")]
    ListTooLong { max: usize, got: usize },
    #[error("Buffer is too small to serialize {ty} into")]
    BufferTooSmall {
        ty: &'static str,
        #[source]
        cause: BufferTooSmall,
    },
    #[error("Buffer is too small to add bit padding specified in {struct_name}")]
    AddPadding {
        struct_name: &'static str,
        #[source]
        cause: BufferTooSmall,
    },
}
