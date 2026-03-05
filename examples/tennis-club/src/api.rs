#![allow(dead_code)]

use crate::{Error, Member, NewMember, UpdateMember};

include!(concat!(env!("OUT_DIR"), "/operations.rs"));
include!(concat!(env!("OUT_DIR"), "/axum_handlers.rs"));
