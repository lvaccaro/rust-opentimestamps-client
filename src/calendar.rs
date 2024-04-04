// Copyright (C) 2024 The OpenTimestamps developers

use ots::hex::Hexed;
use ots::Timestamp;
use reqwest::header::{ACCEPT, USER_AGENT};

const USER_AGENT_OTS: &str = "Rust-OpenTimestamps-Client/0";
const ACCEPT_OTS: &str = "application/vnd.opentimestamps.v1";

pub struct Calendar {
    pub url: String,
}

impl Calendar {
    pub fn submit_calendar(&self, msg: Vec<u8>) -> Result<Timestamp, reqwest::Error> {
        let url = format!("{}/digest", self.url);
        println!("{:?}", url.clone());
        let res = reqwest::blocking::Client::builder()
            .build()
            .unwrap()
            .post(url)
            .header(USER_AGENT, USER_AGENT_OTS)
            .header(ACCEPT, ACCEPT_OTS)
            .body(msg.to_vec())
            .send()
            .unwrap();

        let mut deser = ots::ser::Deserializer::new(res);
        let timestamp = Timestamp::deserialize(&mut deser, msg.to_vec()).unwrap();
        Ok(timestamp)
    }

    pub fn get_timestamp(&self, commitment: Vec<u8>) -> Result<Timestamp, reqwest::Error> {
        let url = format!("{}/timestamp/{}", self.url, Hexed(&commitment));
        println!("{:?}", url);
        let res = reqwest::blocking::Client::builder()
            .build()
            .unwrap()
            .get(url)
            .header(USER_AGENT, USER_AGENT_OTS)
            .header(ACCEPT, ACCEPT_OTS)
            .send()
            .unwrap();

        let mut deser = ots::ser::Deserializer::new(res);
        let timestamp = Timestamp::deserialize(&mut deser, commitment.to_vec()).unwrap();
        Ok(timestamp)
    }
}
