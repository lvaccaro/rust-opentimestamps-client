// Copyright (C) 2024 The OpenTimestamps developers

use opentimestamps::hex::Hexed;
use reqwest::Response;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use std::time::Duration;

#[allow(dead_code)]
pub(crate) const APOOL: &str = "https://a.pool.opentimestamps.org";
#[allow(dead_code)]
pub(crate) const BPOOL: &str = "https://b.pool.opentimestamps.org";
#[allow(dead_code)]
pub(crate) const FINNEY: &str = "https://finney.calendar.eternitywall.com";
#[allow(dead_code)]
pub(crate) const CTLLX: &str = "https://ots.btc.catallaxy.com";

const ACCEPT_OTS: &str = "application/vnd.opentimestamps.v1";
const CONTENT_TYPE_OTS: &str = "application/x-www-form-urlencoded";

pub struct Calendar {
    pub url: String,
    pub timeout: Option<Duration>,
}

impl Calendar {
    pub async fn submit_calendar(&self, msg: Vec<u8>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/digest", self.url);
        reqwest::Client::builder()
            .build()?
            .post(url)
            .header(ACCEPT, ACCEPT_OTS)
            .header(CONTENT_TYPE, CONTENT_TYPE_OTS)
            .body(msg.to_vec())
            .send()
            .await
    }

    pub async fn get_timestamp(&self, commitment: Vec<u8>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/timestamp/{}", self.url, Hexed(&commitment));
        reqwest::Client::builder()
            .build()?
            .get(url)
            .header(ACCEPT, ACCEPT_OTS)
            .send()
            .await
    }
}
