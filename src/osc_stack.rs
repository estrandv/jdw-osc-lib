/*

    Implements the following standard for polling incoming osc messages: 

    OscPoll::init(<url>)
        .on_message("/s_new", msg -> {...})
        .on_tbundle("/queue_notes", msg -> {...})
        .begin() 

*/

use std::collections::{HashMap, HashSet};

use log::warn;
extern crate rosc;

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

use rosc::{OscPacket, OscMessage};

use crate::model::TaggedBundle;

pub struct OSCStack<'a> {
    message_operations: HashMap<String, &'a dyn Fn(OscMessage)>,
    tbundle_operations: HashMap<String, &'a dyn Fn(TaggedBundle)>,
    tbundle_funnels: HashSet<String>,
    host_url: String
}

impl <'a> OSCStack<'a> {
    pub fn init(host_url: String) -> OSCStack<'a> {
        OSCStack {
            message_operations: HashMap::new(),
            tbundle_operations: HashMap::new(),
            tbundle_funnels: HashSet::new(),
            host_url
        }
    }

    pub fn on_message(&'a mut self, tag: &str, operations: &'a dyn Fn(OscMessage)) -> &mut OSCStack {
        self.message_operations.insert(tag.to_string(), operations);
        self
    }

    pub fn on_tbundle(&'a mut self, tag: &str, operations: &'a dyn Fn(TaggedBundle))  -> &mut OSCStack {
        self.tbundle_operations.insert(tag.to_string(), operations);
        self
    }

    // Funnel contents of tagged bundle to be interpreted individually
    // This effectively invalidates any on_tbundle ops for the given bundle tag
    pub fn funnel_tbundle(&'a mut self, tag: &str) -> &mut OSCStack {

        self.tbundle_funnels.insert(tag.to_string());
        self
    }

    fn interpret(&self, packet: OscPacket) {
        match packet {
            OscPacket::Message(osc_msg) => {

                self.message_operations.get(&osc_msg.addr).map(|op| {
                    op(osc_msg);
                });

            },
            OscPacket::Bundle(osc_bundle) => {

                match TaggedBundle::new(&osc_bundle) {
                    Ok(tagged_bundle) => {

                        if self.tbundle_funnels.contains(&tagged_bundle.bundle_tag) {
                            for packet in tagged_bundle.contents {
                                self.interpret(packet);
                            }
                        } else {
                            self.tbundle_operations.get(&tagged_bundle.bundle_tag).map(|op| op(tagged_bundle));
                        }

                    },
                    Err(msg) => warn!("Failed to parse bundle as tagged: {}", msg)
                };
            }
        };

    }

    pub fn begin(&self) {


        let addr = match SocketAddrV4::from_str(&self.host_url) {
            Ok(addr) => addr,
            Err(e) => panic!("{}", e),
        };

        let sock = UdpSocket::bind(addr).unwrap();

        let mut buf = [0u8; 333072];

        loop {


            //let buf = [0u8; rosc::decoder::MTU];
            // TODO: Compare with size in struct declaration (should be same value)
            // THe MTU constant is way too low... I think.
            // Too low results in parts of large packets being dropped before receiving 
            // Heck, might just be some kind of buffer thing where I'm supposed to read 
            // multiple things but only end up reading the first.. . 
            // UPDATE: Found no indication of this in documentation. :c

            match sock.recv_from(&mut buf) {
                Ok((size, _)) => {
                    let (_rem, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();

                    self.interpret(packet);

                }
                Err(e) => {
                    warn!("Failed to receive from socket {}", e);
                }
            };

        }
    }


}