use std::io;

use crate::hid::cmd;
use binary_layout::prelude::*;

binary_layout!(uhid_packet, BigEndian, {
    channel: u32,
    command: u8,
    data: [u8]
});

const FRAME_TYPE_INIT: u8 = 0x80;
const FRAME_TYPE_CONT: u8 = 0x00;

pub struct Packet {
    channel: u32,
    command: Option<crate::hid::cmd::CTapCHIDCmd>,
    seq_no: Option<u8>,
    data: Vec<u8>,
}

impl Packet {
    pub fn new(data: &[u8]) -> Result<Packet, io::Error> {
        let type_or_seq = data[0];

        let view = uhid_packet::View::new(&data[1..data.len()]);
        let channel = view.channel().read();

        let cmd = if type_or_seq & FRAME_TYPE_INIT == FRAME_TYPE_INIT {
            let c = type_or_seq ^ FRAME_TYPE_INIT;
            cmd::CTapCHIDCmd::from_repr(c.into())
        } else {
            None
        };

        Ok(Packet {
            channel: channel,
            command: cmd,
            seq_no: Some(type_or_seq),
            data: data.to_vec(),
        })
    }
}
