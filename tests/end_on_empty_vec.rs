use abstract_bits::{AbstractBits, abstract_bits};

#[abstract_bits]
struct LinkStatusCommand {
    list_length: u5,
    is_first_frame: bool,
    is_last_frame: bool,
    reserved: u1,
    #[abstract_bits(length_from = list_length)]
    link_statuses: Vec<u8>,
}

#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = LinkStatusCommand {
        is_first_frame: false,
        is_last_frame: true,
        link_statuses: Vec::new(),
    }
    .to_abstract_bits()?;
    let link_status_cmd = LinkStatusCommand::from_abstract_bits(&bytes)?;
    assert_eq!(link_status_cmd.link_statuses.len(), 0);
    print!("number of links: {}", link_status_cmd.link_statuses.len());
    Ok(())
}
