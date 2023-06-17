// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc, clippy::multiple_crate_versions)]
#![cfg_attr(feature = "userspace", feature(alloc_error_handler))]

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

extern crate alloc;
#[cfg(feature = "userspace")]
#[macro_use]
extern crate log;

#[cfg(target_arch = "x86_64")]
pub mod osdtentry;
pub mod osvalue;
#[cfg(target_arch = "x86_64")]
pub mod syscall;
#[cfg(feature = "userspace")]
pub mod userspace;

use serde::{Deserialize, Serialize};

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC000_0000;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TKInfo {
    pub identifier: String,
    pub personalities: HashMap<String, HashMap<String, osvalue::OSValue>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TKCache(pub Vec<(TKInfo, Vec<u8>)>);

impl TKCache {
    pub const fn new(inner: Vec<(TKInfo, Vec<u8>)>) -> Self {
        Self(inner)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminationReason {
    Unspecified,
    MalformedArgument,
    MalformedAddress,
    MalformedBody,
    NotFound,
    AlreadyExists,
    InsufficientPermissions,
}
