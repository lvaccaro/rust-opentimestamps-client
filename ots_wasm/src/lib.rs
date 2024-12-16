//extern crate ots_core;
extern crate hex;
extern crate ots_core;
extern crate serde_wasm_bindgen;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

pub mod error;

use error::Error;
use ots_core::client;
use ots_core::opentimestamps::{ser::DigestType, DetachedTimestampFile};

use std::io::BufWriter;
use std::io::Cursor;
use std::io::Write;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BitcoinAttestationResult {
    pub height: u32,
    pub time: u32,
}

#[wasm_bindgen]
pub async fn stamp(digest: String) -> Result<String, Error> {
    let digest = hex::decode(digest).map_err(|_| Error::Generic(String::from("Invalid digest")))?;
    let mut buf = BufWriter::new(Vec::new());
    client::stamps(vec![digest], DigestType::Sha256, None, None)
        .await
        .map_err(|_| Error::Generic(String::from("OTS Stamp error")))
        .unwrap()
        .first()
        .map(|ots| {
            let _ = ots.to_writer(buf.by_ref());
            hex::encode(&buf.into_inner().unwrap())
        })
        .ok_or(Error::Generic(String::from("OTS error")))
}

#[wasm_bindgen]
pub fn info(ots: String) -> Result<String, Error> {
    let bytes = hex::decode(ots).map_err(|_| Error::Generic(String::from("Invalid param")))?;
    let cursor = Cursor::new(bytes);
    let ots = DetachedTimestampFile::from_reader(cursor).unwrap();
    client::info(ots).map_err(|_| Error::Generic(String::from("OTS info error")))
}

#[wasm_bindgen]
pub async fn upgrade(ots: String) -> Result<String, Error> {
    let bytes = hex::decode(ots).map_err(|_| Error::Generic(String::from("Invalid param")))?;
    let cursor = Cursor::new(bytes);
    let mut ots: DetachedTimestampFile = DetachedTimestampFile::from_reader(cursor).unwrap();
    let _ = client::upgrade(&mut ots, None)
        .await
        .map_err(|_| Error::Generic(String::from("OTS upgrade error")));
    let mut buf = BufWriter::new(Vec::new());
    let _ = ots.to_writer(buf.by_ref());
    Ok(hex::encode(&buf.into_inner().unwrap()))
}

#[wasm_bindgen]
pub async fn verify(ots: String) -> Result<BitcoinAttestationResult, Error> {
    let bytes = hex::decode(ots).map_err(|_| Error::Generic(String::from("Invalid param")))?;
    let cursor = Cursor::new(bytes);
    let ots = DetachedTimestampFile::from_reader(cursor).unwrap();
    let att = client::verify(ots, None)
        .await
        .map_err(|_| Error::Generic(String::from("OTS error")))?;
    Ok(BitcoinAttestationResult {
        height: att.height,
        time: att.time,
    })
}
