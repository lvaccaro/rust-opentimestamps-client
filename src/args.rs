use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{fs, path::PathBuf};

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
        #[clap(name = "files", required = true, short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
        files: Vec<Utf8PathBuf>,
    },

    #[clap(long_about = "Upgrade remote calendar timestamps to be locally verifiable")]
    Upgrade {
        /// Existing timestamp(s); moved to FILE.bak
        #[clap(name = "files", required = true, short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
        files: Vec<Utf8PathBuf>,
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
        #[clap(name = "timestamp", required = true)]
        timestamp: Utf8PathBuf,
        /// Specify target file explicitly
        #[clap(name = "target")]
        target: Option<Utf8PathBuf>,
        /// Verify a (hex-encoded) digest rather than a file
        #[clap(name = "digest")]
        digest: Option<String>,
    },
}
