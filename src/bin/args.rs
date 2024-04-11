// Copyright (C) 2024 The OpenTimestamps developers

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use std::time::Duration;

/// A liquid wallet with watch-only confidential descriptors and hardware signers.
/// WARNING: not yet for production use, expect bugs, breaking changes and loss of funds.
#[derive(PartialEq, Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    /// The sub command
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
#[clap(rename_all = "snake")]
pub enum CliCommand {
    #[clap(long_about = "Timestamp files")]
    Stamp {
        /// Filenames
        #[clap(name = "files", required = true, num_args = 1.., value_delimiter = ' ')]
        files: Vec<Utf8PathBuf>,
        /// Create timestamp with the aid of a remote calendar. May be specified multiple times.
        #[clap(name = "calendar_url", short, long)]
        calendar: Option<Vec<String>>,
        /// Timeout before giving up on a calendar.
        #[clap(name = "timeout", short, long)]
        #[arg(value_parser = parse_duration)]
        timeout: Option<Duration>,
    },

    #[clap(long_about = "Upgrade remote calendar timestamps to be locally verifiable")]
    Upgrade {
        /// Existing timestamp(s); moved to FILE.bak
        #[clap(name = "files", required = true, num_args = 1.., value_delimiter = ' ')]
        files: Vec<Utf8PathBuf>,
        /// Override calendars in timestamp
        #[clap(name = "calendar_url", short, long)]
        calendar: Option<Vec<String>>,
    },

    #[clap(long_about = "Show information on a timestamp")]
    Info {
        /// Filename
        #[clap(name = "file", required = true, index = 1)]
        file: Utf8PathBuf,
    },

    #[clap(long_about = "Verify a timestamp")]
    Verify {
        /// Timestamp file
        #[clap(name = "timestamp", required = true, index = 1)]
        timestamp: Utf8PathBuf,
        /// Specify target file explicitly
        #[clap(name = "target", index = 2)]
        target: Option<Utf8PathBuf>,
        /// Verify a (hex-encoded) digest rather than a file
        #[clap(name = "digest", index = 3)]
        digest: Option<String>,
    },
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(std::time::Duration::from_secs(seconds))
}
