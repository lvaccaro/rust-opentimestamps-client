// Copyright (C) 2024 The OpenTimestamps developers

extern crate camino;
extern crate chrono;
extern crate electrum_client;
extern crate env_logger;
extern crate log;
extern crate ots;
extern crate rand;
extern crate reqwest;
extern crate rs_merkle;
extern crate thiserror;

pub mod calendar;
pub mod error;
pub mod extensions;

use calendar::Calendar;
use error::Error;
use extensions::{StepExtension, TimestampExtension};

use chrono::DateTime;
use electrum_client::bitcoin::hashes::Hash;
use electrum_client::{Client, ElectrumApi};
use log::{debug, error, info};
use ots::hex::Hexed;
use ots::ser::DigestType;
use ots::{
    attestation::Attestation,
    op::Op,
    timestamp::{Step, StepData},
    DetachedTimestampFile, Timestamp,
};
use rs_merkle::{algorithms::Sha256, MerkleTree};
use std::convert::TryInto;
use std::time::Duration;

pub fn info(ots: DetachedTimestampFile) -> Result<String, Error> {
    Ok(ots.to_string())
}
/// Verify attestation against a block header
fn verify_against_blockheader(
    digest: [u8; 32],
    block_header: electrum_client::bitcoin::block::Header,
) -> Result<u32, Error> {
    if digest != block_header.merkle_root.to_byte_array() {
        return Err(Error::Generic("Merkle root mismatch".to_string()));
    }
    Ok(block_header.time)
}

fn timestamp_to_date(timestamp: i64) -> String {
    let from = DateTime::from_timestamp(timestamp, 0).unwrap();
    let date = from.naive_local();
    date.format("%Y-%m-%d").to_string()
}

pub struct BitcoinAttestationResult {
    height: usize,
    time: u32,
}
impl std::fmt::Display for BitcoinAttestationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bitcoin block {} attests existence as of {}",
            self.height,
            timestamp_to_date(self.time as i64)
        )
    }
}

pub fn verify(ots: DetachedTimestampFile) -> Result<BitcoinAttestationResult, Error> {
    let client = Client::new("tcp://electrum.blockstream.info:50001").unwrap();
    for attestation in ots.timestamp.all_attestations() {
        match attestation.1 {
            Attestation::Bitcoin { height } => {
                let block_header = client.block_header(height).unwrap();
                debug!("Attestation block hash: {:?}", block_header.block_hash());
                let time =
                    verify_against_blockheader(attestation.0.try_into().unwrap(), block_header)?;
                let result = BitcoinAttestationResult { height, time };
                info!("Success! {}", result);
                return Ok(BitcoinAttestationResult { height, time });
            }
            Attestation::Pending { uri } => {
                debug!("Ignoring Pending Attestation at {:?}", uri);
            }
            Attestation::Unknown { tag: _, data: _ } => {
                debug!("Ignoring Unknown Attestation");
            }
        };
    }
    Err(Error::Generic("No bitcoin attestion found".to_string()))
}

pub fn upgrade(
    ots: &mut DetachedTimestampFile,
    calendar_urls: Option<Vec<String>>,
) -> Result<(), Error> {
    for attestation in ots.timestamp.all_attestations() {
        match attestation.1 {
            Attestation::Bitcoin { height: _ } => {}
            Attestation::Unknown { tag: _, data: _ } => {}
            Attestation::Pending { ref uri } => {
                if calendar_urls
                    .as_ref()
                    .is_some_and(|urls| !urls.contains(uri))
                {
                    error!("No valid calendar found");
                    continue;
                }
                info!("Upgrading to remote calendar {}", uri.to_string());
                let upgraded = upgrade_timestamp(attestation.0, uri.to_string(), None)?;
                ots.timestamp.merge(upgraded);
            }
        };
    }
    Ok(())
}

fn upgrade_timestamp(
    commitment: Vec<u8>,
    calendar_url: String,
    timeout: Option<Duration>,
) -> Result<Timestamp, Error> {
    let res = Calendar {
        url: calendar_url,
        timeout: timeout,
    }
    .get_timestamp(commitment.clone())
    .map_err(|err| Error::NetworkError(err))?;
    let mut deser = ots::ser::Deserializer::new(res);
    Timestamp::deserialize(&mut deser, commitment).map_err(|err| Error::InvalidOts(err))
}

fn timestamp_from_merkle(
    merkle_tree: &MerkleTree<Sha256>,
    leave: [u8; 32],
) -> Result<Timestamp, Error> {
    let index = merkle_tree
        .leaves()
        .unwrap()
        .iter()
        .position(|l| *l == leave)
        .unwrap();
    let proofs = merkle_tree.proof(&[index]);
    //debug!("proofs {:?}", proofs.proof_hashes_hex());
    //debug!("index {:?}", index);

    let mut step = Step {
        data: StepData::Op(Op::Hexlify),
        output: vec![],
        next: vec![],
    };
    let mut digest = leave.to_vec();
    for proof in proofs.proof_hashes().iter().enumerate() {
        let level = proof.0 as u32;
        let odd = (index as i32 / 2_i32.pow(level)) % 2 == 1;
        let op = if odd {
            Op::Prepend(proof.1.to_vec())
        } else {
            Op::Append(proof.1.to_vec())
        };
        let step_pend = Step {
            data: StepData::Op(op.clone()),
            output: op.execute(&digest),
            next: vec![],
        };
        let op = Op::Sha256;
        digest = op.execute(&step_pend.clone().output);
        let step_sha256 = Step {
            data: StepData::Op(op.clone()),
            output: op.execute(&step_pend.clone().output),
            next: vec![],
        };
        if level == 0 {
            step = step_pend;
        } else {
            step.cat(step_pend);
        }
        step.cat(step_sha256);
    }
    Ok(Timestamp {
        start_digest: leave.to_vec(),
        first_step: step,
    })
}

pub fn stamps(
    digests: Vec<Vec<u8>>,
    digest_type: DigestType,
    calendar_urls: Option<Vec<String>>,
    timeout: Option<Duration>,
) -> Result<Vec<DetachedTimestampFile>, Error> {
    let mut merkle_roots: Vec<[u8; 32]> = vec![];
    let mut file_timestamps: Vec<ots::DetachedTimestampFile> = vec![];
    for digest in digests {
        let random: Vec<u8> = (0..16).map(|_| rand::random::<u8>()).collect();
        let nonce_op = ots::op::Op::Append(random);
        let nonce_output_digest = nonce_op.execute(&digest);
        let hash_op = ots::op::Op::Sha256;
        let hash_output_digest = hash_op.execute(&nonce_output_digest);
        let file_timestamp = ots::DetachedTimestampFile {
            digest_type: digest_type,
            timestamp: ots::Timestamp {
                start_digest: digest,
                first_step: Step {
                    data: StepData::Op(nonce_op),
                    output: nonce_output_digest,
                    next: vec![Step {
                        data: StepData::Op(hash_op),
                        output: hash_output_digest.clone(),
                        next: vec![],
                    }],
                },
            },
        };
        //let timestamp = file_timestamp.timestamp;
        file_timestamps.push(file_timestamp.clone());
        merkle_roots.push(hash_output_digest.try_into().unwrap());
    }
    debug!("file_timestamps {}", file_timestamps[0]);
    debug!("merkle_roots {:?}", merkle_roots.len());
    for root in merkle_roots.iter() {
        debug!("{:?}", Hexed(root));
    }
    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&merkle_roots);
    let merkle_tip = merkle_tree.root().unwrap();

    if file_timestamps.len() > 1 {
        for ft in file_timestamps.iter_mut().enumerate() {
            if let Ok(timestamp) = timestamp_from_merkle(&merkle_tree, merkle_roots[ft.0]) {
                ft.1.timestamp.merge(timestamp);
            }
        }
    }
    let calendar_urls = match calendar_urls {
        Some(urls) => urls,
        None => vec![
            calendar::APOOL.to_string(),
            calendar::BPOOL.to_string(),
            calendar::FINNEY.to_string(),
        ],
    };

    let mut calendar_timestamps = vec![];
    for calendar in calendar_urls {
        info!("Submitting to remote calendar {}", calendar);
        let calendar_timestamp = create_timestamp(merkle_tip.to_vec(), calendar.clone(), timeout);
        match calendar_timestamp {
            Ok(timestamp) => calendar_timestamps.push(timestamp),
            Err(e) => error!("Ignoring remote calendar {}: {}", calendar, e.to_string()),
        }
    }
    if calendar_timestamps.is_empty() {
        return Err(Error::Generic("No valid calendar found".to_string()));
    }
    let timestamp: Timestamp;
    if calendar_timestamps.len() == 1 {
        timestamp = calendar_timestamps.first().unwrap().clone();
    } else {
        let steps = calendar_timestamps
            .iter()
            .map(|x| x.first_step.clone())
            .collect();
        let fork = Step {
            data: StepData::Fork,
            output: merkle_tip.to_vec(),
            next: steps,
        };
        timestamp = Timestamp {
            start_digest: merkle_tip.to_vec(),
            first_step: fork,
        };
    }
    for ft in file_timestamps.iter_mut() {
        ft.timestamp.merge(timestamp.clone());
    }
    Ok(file_timestamps)
}

fn create_timestamp(
    stamp: Vec<u8>,
    calendar_url: String,
    timeout: Option<Duration>,
) -> Result<Timestamp, Error> {
    let res = Calendar {
        url: calendar_url,
        timeout: timeout,
    }
    .submit_calendar(stamp.clone())
    .map_err(|err| Error::NetworkError(err))?;
    let mut deser = ots::ser::Deserializer::new(res);
    Timestamp::deserialize(&mut deser, stamp.to_vec()).map_err(|err| Error::InvalidOts(err))
}
