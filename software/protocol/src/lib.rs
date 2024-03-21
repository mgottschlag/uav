#![no_std]

use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Status {
    pub r: f32,
    pub p: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Format, Clone)]
pub struct Command {
    // [0..255]
    pub thrust: [i16; 4],
    // [-90..90]
    pub pose: [i8; 2],
}
