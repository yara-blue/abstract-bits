use abstract_bits::abstract_bits;

#[abstract_bits]
#[derive(Clone)]
struct Type {
    is_maybe_nothing_present: bool,
    #[abstract_bits(presence_from = is_maybe_nothing_present)]
    maybe_nothing: Option<u8>,
}
