// Copyright (C) 2024 The OpenTimestamps developers

extern crate bitcoincore_rpc;
extern crate camino;
extern crate chrono;
extern crate clap;
extern crate electrum_client;
extern crate env_logger;
extern crate log;
extern crate opentimestamps_client;
extern crate ots;
extern crate rand;
extern crate reqwest;
extern crate rs_merkle;

mod args;

use crate::args::*;
use bitcoincore_rpc::{Auth, Client};
use camino::Utf8PathBuf;
use clap::Parser;
use electrum_client::bitcoin::hex::FromHex;
use log::{debug, error, info};
use opentimestamps_client::error::Error;
use ots::hex::Hexed;
use ots::ser::DigestType;
use ots::{op::Op, DetachedTimestampFile};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::{fs, io::Read};

fn main() {
    env_logger::init();

    let cli_opts: CliOpts = CliOpts::parse();

    match handle_command(cli_opts) {
        Ok(_result) => {}
        Err(e) => error!("{}", e.to_string()),
    }
}

pub(crate) fn handle_command(cli_opts: CliOpts) -> Result<(), Error> {
    let result = match cli_opts.command {
        CliCommand::Info { file } => info(file),
        CliCommand::Stamp {
            files,
            calendar,
            timeout,
        } => stamps(files, calendar, timeout),
        CliCommand::Upgrade { files, calendar } => upgrade(files, calendar),
        CliCommand::Verify {
            target,
            digest,
            timestamp,
        } => verify(target, digest, timestamp, cli_opts.bitcoin),
    };
    result.map_err(|e| e.into())
}

fn info(file: Utf8PathBuf) -> Result<(), Error> {
    let fh = fs::File::open(file).map_err(|_| Error::InvalidFile)?;
    let ots = DetachedTimestampFile::from_reader(fh).map_err(|err| Error::InvalidOts(err))?;
    print!("{}", opentimestamps_client::info(ots)?);
    Ok(())
}

fn file_digest(path: Utf8PathBuf, digest_type: DigestType) -> Result<Vec<u8>, Error> {
    let mut fh = fs::File::open(path).map_err(|_| Error::InvalidFile)?;
    let mut buffer = Vec::new();
    fh.read_to_end(&mut buffer).map_err(|_| Error::IOError)?;
    let op = match digest_type {
        DigestType::Sha1 => Op::Sha1,
        DigestType::Sha256 => Op::Sha256,
        DigestType::Ripemd160 => Op::Ripemd160,
    };
    Ok(op.execute(&buffer))
}

fn stamps(
    files: Vec<Utf8PathBuf>,
    calendar_urls: Option<Vec<String>>,
    timeout: Option<Duration>,
) -> Result<(), Error> {
    let mut file_digests = vec![];
    let digest_type = DigestType::Sha256;
    for file in files.clone() {
        file_digests.push(file_digest(file, digest_type)?);
    }
    let timestamps =
        opentimestamps_client::stamps(file_digests, digest_type, calendar_urls, timeout)?;
    for (in_file, ots) in files.iter().zip(timestamps) {
        let timestamp_file_path = format!("{}.ots", in_file);
        let file = fs::File::create(timestamp_file_path).map_err(|_| Error::InvalidFile)?;
        ots.to_writer(file).map_err(|_| Error::IOError)?;
    }
    Ok(())
}

fn upgrade(files: Vec<Utf8PathBuf>, calendar_urls: Option<Vec<String>>) -> Result<(), Error> {
    for file in files {
        upgrade_file(file, calendar_urls.clone())?;
    }
    Ok(())
}

fn upgrade_file(path: Utf8PathBuf, calendar_urls: Option<Vec<String>>) -> Result<(), Error> {
    debug!("Upgrading {}", path);

    let file = fs::File::open(path.clone()).map_err(|_| Error::InvalidFile)?;
    let mut ots = DetachedTimestampFile::from_reader(file).map_err(|err| Error::InvalidOts(err))?;
    opentimestamps_client::upgrade(&mut ots, calendar_urls)?;

    let backup_name = format!("{}.bak", path);
    debug!(
        "Got new timestamp data; renaming existing timestamp to {}",
        backup_name
    );
    if Path::new(backup_name.as_str()).exists() {
        error!("Could not backup timestamp: {} already exists", backup_name);
        return Ok(());
    }
    fs::rename(path.clone(), backup_name).map_err(|_| Error::InvalidFile)?;

    let file = fs::File::create(path).map_err(|_| Error::InvalidFile)?;
    ots.to_writer(file).map_err(|_| Error::IOError)?;

    Ok(())
}

fn verify(
    target: Option<Utf8PathBuf>,
    digest: Option<String>,
    timestamp: Utf8PathBuf,
    bitcoin: Option<BitcoinOpts>,
) -> Result<(), Error> {
    let file = fs::File::open(timestamp.clone()).map_err(|_| Error::InvalidFile)?;
    let mut detached_timestamp =
        DetachedTimestampFile::from_reader(file).map_err(|err| Error::InvalidOts(err))?;

    if let Some(digest) = digest {
        let bytes = Vec::<u8>::from_hex(&digest.as_str()).map_err(|_| Error::IOError)?;
        if bytes != detached_timestamp.timestamp.start_digest {
            let msg = format!(
                "Digest provided does not match digest in timestamp, {:?} {:?}",
                Hexed(detached_timestamp.timestamp.start_digest.by_ref()),
                detached_timestamp.digest_type
            );
            return Err(Error::Generic(msg));
        }
    } else {
        let target_filename = match target {
            Some(target) => target,
            None => target_filename(timestamp)?,
        };
        debug!(
            "Hashing file, algorithm {:?}",
            detached_timestamp.digest_type
        );
        let actual_file_digest = file_digest(target_filename, detached_timestamp.digest_type)?;
        debug!(
            "Got digest {:?}",
            Hexed(detached_timestamp.timestamp.start_digest.by_ref())
        );

        if actual_file_digest != detached_timestamp.timestamp.start_digest {
            let msg = format!(
                "Expected digest {:?}",
                Hexed(detached_timestamp.timestamp.start_digest.by_ref())
            );
            return Err(Error::Generic(msg));
        }
    }
    let bitcoin_client = match bitcoin {
        Some(opts) => Some(
            Client::new(
                opts.bitcoin_node
                    .unwrap_or("localhost".to_string())
                    .as_str(),
                Auth::UserPass(
                    opts.bitcoin_username.unwrap(),
                    opts.bitcoin_password.unwrap(),
                ),
            )
            .map_err(|_| Error::BitcoinNodeError)?,
        ),
        None => None,
    };
    let attestation = opentimestamps_client::verify(detached_timestamp, bitcoin_client)?;
    info!("Success! {}", attestation);
    Ok(())
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
