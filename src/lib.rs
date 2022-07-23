#![feature(result_flattening)]
use rosc::{OscBundle, OscMessage, OscPacket};

/*
    OSC structs for careful parsing and management of expected message and bundle types.
 */

use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use rosc::{OscError, OscType};
use std::option::Option;
use std::sync::{Arc, Mutex};

/*
    Adding some convenience functions for OscMessage args
 */
pub trait OscArgHandler {
    fn expect_addr(&self, addr_name: &str) -> Result<(), String>;
    fn expect_args(&self, amount: usize) -> Result<String, String>;
    fn get_string_at(&self, index: usize, name: &str, ) -> Result<String, String>;
    fn get_float_at(&self, index: usize, name: &str, ) -> Result<f32, String>;
    fn get_int_at(&self, index: usize, name: &str, ) -> Result<i32, String>;
}

impl OscArgHandler for OscMessage {

    fn expect_addr(&self, addr_name: &str) -> Result<(), String> {
        if &self.addr.to_string() != addr_name {
            return Err(format!("Attempted to format {} as the wrong kind of message - this likely a human error in the source code", addr_name));
        }

        Ok(())
    }

    fn expect_args(&self, amount: usize) -> Result<String, String> {

        if self.args.len() < amount {
            return Err(format!("Message did not contain the {} first required args.", amount));
        }

        Ok("Ok".to_string())
    }

    fn get_string_at(&self, index: usize, name: &str, ) -> Result<String, String> {
        let err_msg = format!("{} string not found as {}th arg", name, index);
        self.args
            .get(index)
            .map_or(None, |some| some.clone().string())
            .map_or(Err(err_msg), |s| Ok(s))
    }

    fn get_float_at(&self, index: usize, name: &str, ) -> Result<f32, String> {
        let err_msg = format!("{} float not found as {}th arg", name, index);
        self.args
            .get(index)
            .map_or(None, |some| some.clone().float())
            .map_or(Err(err_msg), |s| Ok(s))
    }

    fn get_int_at(&self, index: usize, name: &str, ) -> Result<i32, String> {
        let err_msg = format!("{} float not found as {}th arg", name, index);
        self.args
            .get(index)
            .map_or(None, |some| some.clone().int())
            .map_or(Err(err_msg), |s| Ok(s))
    }

}


/*
    In order to properly utilize bundles I have created a standard where the first
        packet in every JDW-compatible bundle is an OSC message with a bundle type
        string contained within, e.g.: ["/bundle_tag", "nrt_record_request"]
 */
#[derive(Debug)]
pub struct TaggedBundle {
    pub bundle_tag: String,
    pub contents: Vec<OscPacket>
}

impl TaggedBundle {
    pub fn new(bundle: &OscBundle) -> Result<TaggedBundle, String> {
        let first_msg = match bundle.content.get(0).ok_or("Empty bundle")?.clone() {
            OscPacket::Message(msg) => { Option::Some(msg) }
            OscPacket::Bundle(_) => {Option::None}
        }.ok_or("First element in bundle not an info message!")?;

        if first_msg.addr != "/bundle_info" {
            return Err(format!("Expected /bundle_info as first message in bundle, got: {}", &first_msg.addr));
        }

        let bundle_tag = first_msg.args.get(0)
            .ok_or("bundle info empty")?
            .clone()
            .string().ok_or("bundle info should be a string")?;

        let contents = if bundle.content.len() > 1 {(&bundle.content[1..].to_vec()).clone()} else {vec![]};

        Ok(TaggedBundle {
            bundle_tag,
            contents
        })
    }

    fn get_packet(&self, content_index: usize) -> Result<OscPacket, String> {
        self.contents.get(content_index)
            .map(|pct| pct.clone())
            .ok_or("Failed to fetch packet".to_string())
    }

    fn get_message(&self, content_index: usize) -> Result<OscMessage, String> {
        self.contents.get(content_index)
            .map(|pct| pct.clone())
            .ok_or("Invalid index".to_string())
            .map(|pct| match pct {
                OscPacket::Message(msg) => {
                    Ok(msg)
                }
                _ => {Err("Not a message".to_string())}
            })
            .flatten()
    }

    fn get_bundle(&self, content_index: usize) -> Result<OscBundle, String> {
        self.contents.get(content_index)
            .map(|pct| pct.clone())
            .ok_or("Invalid index".to_string())
            .map(|pct| match pct {
                OscPacket::Bundle(msg) => {
                    Ok(msg)
                }
                _ => {Err("Not a bundle".to_string())}
            })
            .flatten()
    }
}

/*
    Timed osc packets are packets with a relative float time tag.
    Used for all kinds of arbitrary ordering, such as relative execution time in a sequence.
    [/bundle_info, "timed_msg"]
    [/timed_msg_info, 0.0]
    [... packet ...]
 */
#[derive(Debug, Clone)]
pub struct TimedOSCPacket {
    pub time: f32,
    pub packet: OscPacket,
}

impl TimedOSCPacket {

    pub fn from_bundle(bundle: TaggedBundle) -> Result<TimedOSCPacket, String>{
        if &bundle.bundle_tag != "timed_msg" {
            return Err(format!("Attempted to parse {} as timed_msg bundle", &bundle.bundle_tag));
        }

        let info_msg = bundle.get_message(0)?;
        let packet = bundle.get_packet(1)?;

        info_msg.expect_addr("/timed_msg_info")?;
        let time = info_msg.get_float_at(0, "time")?;

        Ok(TimedOSCPacket {
            time,
            packet
        })

    }
}
