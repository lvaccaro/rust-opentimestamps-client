// Copyright (C) 2024 The OpenTimestamps developers

use ots::hex::Hexed;
use reqwest::blocking::Response;
use reqwest::header::{ACCEPT, USER_AGENT};

#[allow(dead_code)]
pub(crate) const APOOL: &str = "https://a.pool.opentimestamps.org";
#[allow(dead_code)]
pub(crate) const BPOOL: &str = "https://b.pool.opentimestamps.org";
#[allow(dead_code)]
pub(crate) const FINNEY: &str = "https://finney.calendar.eternitywall.com";
#[allow(dead_code)]
pub(crate) const CTLLX: &str = "https://ots.btc.catallaxy.com";

const USER_AGENT_OTS: &str = "Rust-OpenTimestamps-Client/0";
const ACCEPT_OTS: &str = "application/vnd.opentimestamps.v1";

pub struct Calendar {
    pub url: String,
}

impl Calendar {
    pub fn submit_calendar(&self, msg: Vec<u8>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/digest", self.url);
        println!("{:?}", url.clone());
        reqwest::blocking::Client::builder()
            .build()?
            .post(url)
            .header(USER_AGENT, USER_AGENT_OTS)
            .header(ACCEPT, ACCEPT_OTS)
            .body(msg.to_vec())
            .send()
    }

    pub fn get_timestamp(&self, commitment: Vec<u8>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/timestamp/{}", self.url, Hexed(&commitment));
        println!("{:?}", url);
        reqwest::blocking::Client::builder()
            .build()?
            .get(url)
            .header(USER_AGENT, USER_AGENT_OTS)
            .header(ACCEPT, ACCEPT_OTS)
            .send()
    }
}
