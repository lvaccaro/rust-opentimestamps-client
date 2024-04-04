extern crate camino;
extern crate chrono;
extern crate clap;
extern crate electrum_client;
extern crate env_logger;
extern crate log;
extern crate ots;
extern crate rand;
extern crate reqwest;
extern crate rs_merkle;

mod args;
mod calendar;
mod step_extension;
mod timestamp_extension;

use crate::args::*;
use camino::Utf8PathBuf;
use chrono::DateTime;
use clap::{Error, Parser};
use electrum_client::bitcoin::hashes::Hash;
use electrum_client::bitcoin::hex::FromHex;
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
use rs_merkle::{
    algorithms::Sha256, MerkleTree,
};
use std::io::Write;
use std::path::Path;
use std::{
    convert::TryInto,
    fs,
    io::Read,
};

use calendar::*;
use step_extension::*;
use timestamp_extension::*;

fn main() {
    env_logger::init();

    let cli_opts: CliOpts = CliOpts::parse();

    match handle_command(cli_opts) {
        Ok(result) => println!("{}", result),
        Err(e) => println!("error {}", e.to_string()),
    }
}

pub(crate) fn handle_command(cli_opts: CliOpts) -> Result<String, Error> {
    let result = match cli_opts.command {
        CliCommand::Info { file } => info(file),
        CliCommand::Stamp { files } => stamps(files),
        CliCommand::Upgrade { files } => upgrade(files),
        CliCommand::Verify {
            target,
            digest,
            timestamp,
        } => verify(target, digest, timestamp),
    };
    result.map_err(|e| e.into())
}

fn info(file: Utf8PathBuf) -> Result<String, Error> {
    let fh = fs::File::open(file).unwrap();
    let ots = DetachedTimestampFile::from_reader(fh).unwrap();
    Ok(ots.to_string())
}

fn file_digest(path: Utf8PathBuf, digest_type: DigestType) -> Result<Vec<u8>, Error> {
    let mut fh = fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    fh.read_to_end(&mut buffer)?;
    let op = match digest_type {
        DigestType::Sha1 => Op::Sha1,
        DigestType::Sha256 => Op::Sha256,
        DigestType::Ripemd160 => Op::Ripemd160,
    };
    Ok(op.execute(&buffer))
}

fn stamps(files: Vec<Utf8PathBuf>) -> Result<String, Error> {
    let mut merkle_roots: Vec<[u8; 32]> = vec![];
    let mut file_timestamps: Vec<ots::DetachedTimestampFile> = vec![];
    for file in files.clone() {
        let file_digest = file_digest(file, DigestType::Sha256)?;

        let random: Vec<u8> = (0..16).map(|_| rand::random::<u8>()).collect();
        let nonce_op = ots::op::Op::Append(random);
        let nonce_output_digest = nonce_op.execute(&file_digest);
        let hash_op = ots::op::Op::Sha256;
        let hash_output_digest = hash_op.execute(&nonce_output_digest);
        let mut file_timestamp = ots::DetachedTimestampFile {
            digest_type: ots::ser::DigestType::Sha256,
            timestamp: ots::Timestamp {
                start_digest: file_digest,
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
    println!("file_timestamps {}", file_timestamps[0]);
    println!("merkle_roots {:?}", merkle_roots.len());
    for root in merkle_roots.iter() {
        println!("{:?}", Hexed(root));
    }
    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&merkle_roots);
    let merkle_tip = merkle_tree.root().unwrap();

    for ft in file_timestamps.iter_mut().enumerate() {
        if let Ok(timestamp) = timestamp_from_merkle(&merkle_tree, merkle_roots[ft.0]) {
            ft.1.timestamp.merge(timestamp);
        }
    }

    let calendar_url = "https://finney.calendar.eternitywall.com";
    let calendar_timestamp =
        create_timestamp(merkle_tip.to_vec(), calendar_url.to_string()).unwrap();

    for ft in file_timestamps.iter_mut() {
        ft.timestamp.merge(calendar_timestamp.clone());
    }

    for (in_file, ots) in files.iter().zip(file_timestamps) {
        let timestamp_file_path = format!("{}.ots", in_file);
        let file = fs::File::create(timestamp_file_path).unwrap();
        println!("{:?}", ots);
        ots.to_writer(file);
    }

    Ok("".to_string())
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
    //println!("proofs {:?}", proofs.proof_hashes_hex());
    //println!("index {:?}", index);

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

fn upgrade(files: Vec<Utf8PathBuf>) -> Result<String, Error> {
    for file in files {
        upgrade_file(file).unwrap();
    }
    Ok("".to_string())
}

fn upgrade_file(path: Utf8PathBuf) -> Result<String, Error> {
    debug!("Upgrading {}", path);

    let file = fs::File::open(path.clone()).unwrap();
    let mut ots = DetachedTimestampFile::from_reader(file).unwrap();
    let changed = upgrade_timestamp(&ots.timestamp).unwrap();
    ots.timestamp = changed;

    let backup_name = format!("{}.bak", path);
    debug!("Got new timestamp data; renaming existing timestamp to {}", backup_name);
    if Path::new(backup_name.as_str()).exists() {
        error!("Could not backup timestamp: {} already exists", backup_name);
        return Ok("".to_string());
    }
    fs::rename(path.clone(), backup_name).unwrap();

    let file = fs::File::create(path).unwrap();
    ots.to_writer(file).unwrap();

    Ok("".to_string())
}

fn upgrade_timestamp(timestamp: &Timestamp) -> Result<Timestamp, reqwest::Error> {
    let mut timestamp = timestamp.clone();
    for attestation in timestamp.all_attestations() {
        let calendar_url = match attestation.1 {
            Attestation::Pending { ref uri } => Some(uri.clone()),
            Attestation::Bitcoin { height } => None,
            Attestation::Unknown { ref tag, ref data } => None,
        };
        let calendar = Calendar {
            url: calendar_url.unwrap(),
        };
        let upgraded = calendar.get_timestamp(attestation.0);
        timestamp.merge(upgraded.unwrap());
    }
    Ok(timestamp)
}

fn create_timestamp(stamp: Vec<u8>, calendar_url: String) -> Result<Timestamp, reqwest::Error> {
    info!("Submitting to remote calendar {}", calendar_url);
    Calendar { url: calendar_url }.submit_calendar(stamp)
}

fn verify(
    target: Option<Utf8PathBuf>,
    digest: Option<String>,
    timestamp: Utf8PathBuf,
) -> Result<String, Error> {
    let file = fs::File::open(timestamp.clone()).unwrap();
    let mut detached_timestamp = DetachedTimestampFile::from_reader(file).unwrap();

    if let Some(digest) = digest {
        let bytes = Vec::<u8>::from_hex(&digest.as_str()).unwrap();
        if bytes != detached_timestamp.timestamp.start_digest {
            error!(
                "Digest provided does not match digest in timestamp, {:?} {:?}",
                Hexed(detached_timestamp.timestamp.start_digest.by_ref()),
                detached_timestamp.digest_type
            );
            return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
        }
    } else {
        let target_filename = match target {
            Some(target) => target,
            None => target_filename(timestamp).unwrap(),
        };
        debug!(
            "Hashing file, algorithm {:?}",
            detached_timestamp.digest_type
        );
        let actual_file_digest =
            file_digest(target_filename, detached_timestamp.digest_type).unwrap();
        debug!(
            "Got digest {:?}",
            Hexed(detached_timestamp.timestamp.start_digest.by_ref())
        );

        if actual_file_digest != detached_timestamp.timestamp.start_digest {
            debug!(
                "Expected digest {:?}",
                Hexed(detached_timestamp.timestamp.start_digest.by_ref())
            );
            error!("File does not match original!");
            return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
        }
    }
    verify_timestamp(detached_timestamp.timestamp)
}

fn target_filename(timestamp: Utf8PathBuf) -> Result<Utf8PathBuf, Error> {
    // Target not specified, so assume it's the same name as the
    // timestamp file minus the .ots extension.
    assert!(timestamp.file_name().unwrap().ends_with(".ots"));

    let mut target = timestamp.clone();
    let target_filename = timestamp.file_name().unwrap().strip_suffix(".ots").unwrap();
    info!("Assuming target filename is {}", target_filename);

    target.pop();
    target.push(Utf8PathBuf::from(target_filename));
    Ok(target)
}

fn verify_timestamp(timestamp: Timestamp) -> Result<String, Error> {
    let mut client = Client::new("tcp://electrum.blockstream.info:50001").unwrap();
    for attestation in timestamp.all_attestations() {
        match attestation.1 {
            Attestation::Bitcoin { height } => {
                let block_header = client.block_header(height).unwrap();
                debug!("Attestation block hash: {:?}", block_header.block_hash());
                match verify_against_blockheader(
                    attestation.1,
                    attestation.0.try_into().unwrap(),
                    block_header,
                ) {
                    Ok(time) => {
                        info!(
                            "Success! Bitcoin block {} attests existence as of {}",
                            height,
                            timestamp_to_date(time as i64)
                        );
                        return Ok("".to_string());
                    }
                    Err(e) => {
                        error!("Bitcoin verification failed: {:?}", e.to_string());
                        return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
                    }
                }
            }
            Attestation::Pending { uri } => {
                debug!("Ignoring Pending Attestation at {:?}", uri);
            }
            Attestation::Unknown { tag, data } => {
                debug!("Ignoring Unknown Attestation");
            }
        };
    }
    Ok("".to_string())
}

/// Verify attestation against a block header
fn verify_against_blockheader(
    attestation: Attestation,
    digest: [u8; 32],
    block_header: electrum_client::bitcoin::block::Header,
) -> Result<u32, Error> {
    if digest != block_header.merkle_root.to_byte_array() {
        return Err(Error::new(clap::error::ErrorKind::ValueValidation));
    }
    Ok(block_header.time)
}

fn timestamp_to_date(timestamp: i64) -> String {
    let from = DateTime::from_timestamp(timestamp, 0).unwrap();
    let date = from.naive_local();
    date.format("%Y-%m-%d").to_string()
}
