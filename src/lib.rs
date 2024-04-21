// Copyright (C) 2024 The OpenTimestamps developers

extern crate bitcoincore_rpc;
extern crate camino;
extern crate chrono;
extern crate electrum_client;
extern crate env_logger;
extern crate log;
extern crate opentimestamps;
extern crate rand;
extern crate reqwest;
extern crate rs_merkle;
extern crate thiserror;

pub mod calendar;
pub mod error;
pub mod extensions;
pub mod client;

uniffi::include_scaffolding!("ots");
