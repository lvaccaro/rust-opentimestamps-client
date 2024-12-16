extern crate ots_core;
extern crate uniffi;

use ots_core::client;
use ots_core::client::BitcoinAttestationResult;
use ots_core::error::Error as OtsError;
use ots_core::opentimestamps::{ser::DigestType, DetachedTimestampFile};
use std::io::Cursor;
use std::time::Duration;

pub fn stamps(
    digests: Vec<Vec<u8>>,
    digest_type: DigestType,
    calendar_urls: Option<Vec<String>>,
    timeout: Option<u64>,
) -> Result<Vec<Vec<u8>>, OtsError> {
    let otss = client::stamps(
        digests,
        digest_type,
        calendar_urls,
        Some(Duration::from_secs(timeout.unwrap_or(5))),
    )?;
    let mut buffers = vec![vec![]];
    for (buffer, ots) in buffers.iter_mut().zip(otss) {
        ots.to_writer(buffer).unwrap();
    }
    Ok(buffers)
}

pub fn info(ots: Vec<u8>) -> Result<String, OtsError> {
    let cursor = Cursor::new(ots);
    let ots = DetachedTimestampFile::from_reader(cursor).unwrap();
    client::info(ots)
}

pub fn upgrade(ots: Vec<u8>, calendar_urls: Option<Vec<String>>) -> Result<Vec<u8>, OtsError> {
    let cursor = Cursor::new(ots);
    let mut ots = DetachedTimestampFile::from_reader(cursor).unwrap();
    client::upgrade(&mut ots, calendar_urls)?;
    let mut buffer = vec![];
    ots.to_writer(&mut buffer).unwrap();
    Ok(buffer)
}

pub fn verify(ots: Vec<u8>) -> Result<BitcoinAttestationResult, OtsError> {
    let cursor = Cursor::new(ots);
    let ots = DetachedTimestampFile::from_reader(cursor).unwrap();
    client::verify(ots, None)
}

uniffi::include_scaffolding!("ots");
