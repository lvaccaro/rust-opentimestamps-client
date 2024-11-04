// Copyright (C) 2024 The OpenTimestamps developers

extern crate bitcoincore_rpc;
extern crate chrono;
extern crate electrum_client;
extern crate env_logger;
extern crate log;
pub extern crate opentimestamps;
extern crate rand;
extern crate reqwest;
extern crate rs_merkle;
extern crate thiserror;

pub mod client;
pub mod error;
pub mod extensions;

#[cfg(feature = "blocking")]
pub mod block_calendar;

#[cfg(feature = "async")]
pub mod async_calendar;
