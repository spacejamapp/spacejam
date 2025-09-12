//! Fuzz tests

use spacejam::fuzz::message::Message;

include!(concat!(env!("OUT_DIR"), "/fuzz.rs"));
