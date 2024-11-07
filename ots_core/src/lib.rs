// Copyright (C) 2024 The OpenTimestamps developers

extern crate bitcoincore_rpc;
extern crate bitcoin_hashes;
extern crate chrono;
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


#[cfg(all(feature = "blocking", not(feature = "async")))]
extern crate electrum_client;

#[cfg(all(feature = "async", not(feature = "blocking")))]
extern crate esplora_client;

#[cfg(all(feature = "blocking", not(feature = "async")))]
pub mod block_calendar;

#[cfg(all(feature = "async", not(feature = "blocking")))]
pub mod async_calendar;
