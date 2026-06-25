use abstract_bits::{AbstractBits, abstract_bits};
/// These tests where taken and adapted from the `zigpy/ziggurat` project.

/// Zigbee spec 3.4.2 Route Reply Command
#[abstract_bits]
#[derive(Debug, PartialEq)]
pub struct NwkRouteReplyCommand {
    reserved: u4,
    has_originator_eui64: bool,
    has_responder_eui64: bool,
    reserved: u2,
    pub route_request_identifier: u8,
    pub originator_nwk: Nwk,
    pub responder_nwk: Nwk,
    pub path_cost: u8,
    #[abstract_bits(presence_from = has_originator_eui64)]
    pub originator_eui64: Option<Eui64>,
    #[abstract_bits(presence_from = has_responder_eui64)]
    pub responder_eui64: Option<Eui64>,
}

#[abstract_bits]
#[derive(Debug, Eq, PartialEq)]
pub struct Eui64(pub [u8; 8]);

impl Eui64 {
    pub fn from_hex(text: &str) -> Self {
        // Strip off colons and a 0x prefix, if present
        let text = text.replace(":", "").replace("0x", "");

        if text.len() != 16 {
            panic!("Invalid Eui64 length");
        }

        let mut eui64 = [0; 8];
        hex::decode_to_slice(text, &mut eui64).expect("Decoding failed");

        eui64.reverse();

        Self(eui64)
    }
}

#[abstract_bits]
#[derive(Debug, Eq, PartialEq)]
pub struct Nwk(pub u16);

#[test]
fn test_nwk_route_reply_command() {
    let bytes: [u8; 23] = [
        48, 95, 55, 95, 10, 147, 3, 113, 56, 33, 5, 1, 136, 23, 0, 174, 211, 31, 11, 1,
        136, 23, 0,
    ];

    dbg!(NwkRouteReplyCommand::MAX_BITS);
    dbg!(bytes.len());
    let command = NwkRouteReplyCommand::from_abstract_bytes(&bytes).unwrap();

    assert_eq!(
        command,
        NwkRouteReplyCommand {
            route_request_identifier: 95,
            originator_nwk: Nwk(0x5F37),
            responder_nwk: Nwk(0x930A),
            path_cost: 3,
            originator_eui64: Some(dbg!(Eui64::from_hex("00:17:88:01:05:21:38:71"))),
            responder_eui64: Some(dbg!(Eui64::from_hex("00:17:88:01:0b:1f:d3:ae"))),
        }
    );

    assert_eq!(command.to_abstract_bytes().unwrap(), &bytes);
}

/// Zigbee spec compressed: 3.4.8.3
#[abstract_bits]
#[derive(Debug, PartialEq)]
pub struct NwkLinkStatusCommand {
    link_statuses_len: u5,
    pub is_first_frame: bool,
    pub is_last_frame: bool,
    reserved: u1,
    #[abstract_bits(length_from = link_statuses_len)]
    pub link_statuses: Vec<NwkLinkStatus>,
}

/// Zigbee spec 3.4.8
#[abstract_bits]
#[derive(Debug, PartialEq)]
pub struct NwkLinkStatus {
    pub address: Nwk,
    pub incoming_cost: u3,
    reserved: u1,
    pub outgoing_cost: u3,
    reserved: u1,
}

#[test]
fn test_nwk_link_status_command() {
    use hex_literal::hex;
    let bytes = hex!("0862e73c120ac711").to_vec();
    let command = NwkLinkStatusCommand::from_abstract_bytes(&bytes[1..]).unwrap();

    assert_eq!(
        command,
        NwkLinkStatusCommand {
            is_first_frame: true, // byte 0x62 -> 0b01100010
            is_last_frame: true,
            link_statuses: vec![
                NwkLinkStatus {
                    address: Nwk(0x3CE7), // e7 3c
                    incoming_cost: 2,     // 12 -> 0b00010010 (inc=2, out=1)
                    outgoing_cost: 1,
                },
                NwkLinkStatus {
                    address: Nwk(0xC70A), // 0a c7
                    incoming_cost: 1,     // 11 -> 0b00010001 (inc=1, out=1)
                    outgoing_cost: 1,
                },
            ],
        }
    );

    assert_eq!(&command.to_abstract_bytes().unwrap(), &bytes[1..]);
}

#[abstract_bits(bits = 2)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum NwkFrameType {
    Data = 0b00,
    Command = 0b01,
    Interpan = 0b11,
}

#[abstract_bits(bits = 2)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum NwkRouteDiscovery {
    Suppress = 0b00,
    Enable = 0b01,
    WithMulticast = 0b10,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
pub struct NwkFrameControl {
    pub frame_type: NwkFrameType,
    pub protocol_version: u4,
    pub discover_route: NwkRouteDiscovery,
    pub multicast: bool,
    pub security: bool,
    pub source_route: bool,
    pub destination: bool,
    pub extended_source: bool,
    pub end_device_initiator: bool,
    reserved: u2,
}

#[abstract_bits]
#[derive(Debug, PartialEq, Eq)]
pub struct NwkSourceRoute {
    relay_count: u8,
    pub relay_index: u8,
    #[abstract_bits(length_from = relay_count)]
    pub relays: Vec<Nwk>,
}

#[abstract_bits]
#[derive(Debug, PartialEq)]
pub struct NwkHeader {
    pub frame_control: NwkFrameControl,

    pub destination: Nwk,
    pub source: Nwk,
    pub radius: u8,
    pub sequence_number: u8,

    #[abstract_bits(presence_from = frame_control.destination)]
    pub destination_ieee: Option<Eui64>,
    #[abstract_bits(presence_from = frame_control.extended_source)]
    pub source_ieee: Option<Eui64>,
    #[abstract_bits(presence_from = frame_control.multicast)]
    pub multicast_control: Option<u8>,
    #[abstract_bits(presence_from = frame_control.source_route)]
    pub source_route: Option<NwkSourceRoute>,
}

#[test]
fn test_nested_nwk_header() {
    use hex_literal::hex;
    let bytes = hex!("0806e73c375f1dcc010039f9").to_vec();
    let header = NwkHeader::from_abstract_bytes(&bytes).unwrap();

    assert_eq!(
        header,
        NwkHeader {
            frame_control: NwkFrameControl {
                frame_type: NwkFrameType::Data,
                protocol_version: 2,
                discover_route: NwkRouteDiscovery::Suppress,
                multicast: false,
                security: true,
                source_route: true,
                destination: false,
                extended_source: false,
                end_device_initiator: false,
            },
            destination: Nwk(0x3ce7),
            source: Nwk(0x5f37),
            radius: 29,
            sequence_number: 204,
            destination_ieee: None,
            source_ieee: None,
            multicast_control: None,
            source_route: Some(NwkSourceRoute {
                relay_index: 0,
                relays: vec![Nwk(0xf939)],
            }),
        }
    );

    assert_eq!(&header.to_abstract_bytes().unwrap(), &bytes);
}
