use num_enum::{IntoPrimitive, TryFromPrimitive};
use smallvec::SmallVec;
use std::fmt::Formatter;
use std::str::FromStr;
use std::{
    fmt::{Debug, Display},
    ptr,
};
use strum::{Display, EnumIter, EnumString};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct SubPar<'s>(pub &'s Subject, pub &'s Parallel);

#[derive(Debug, Clone)]
pub struct Subject {
    pub code: String,
    pub name: String,

    pub lectures: Vec<Parallel>,
    pub seminars: Vec<Parallel>,
    pub labs: Vec<Parallel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parallel {
    pub time: SmallVec<[Time; 4]>,
    pub kind: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, EnumIter, Display)]
#[repr(u8)]
pub enum Day {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub offset: u8,
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:02}:{:02}", self.offset / 4, (self.offset % 4) * 15)
    }
}

#[derive(Error, Debug)]
pub enum BlockParseError {
    #[error("Invalid format")]
    InvalidFormat(),

    #[error("Invalid time")]
    InvalidTime(),
}

impl FromStr for Block {
    type Err = BlockParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (h, m) = s.split_once(':').ok_or(BlockParseError::InvalidFormat())?;
        let time = h
            .parse::<u32>()
            .ok()
            .zip(m.parse::<u32>().ok())
            .map(|(h, m)| h * 60 + m)
            .filter(|&t| t % 15 == 0)
            .ok_or(BlockParseError::InvalidFormat())?;

        Ok(Self {
            offset: (time / 15) as u8,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Week {
    #[strum(serialize = "S", to_string = "Even")]
    S,
    #[strum(serialize = "L", to_string = "Odd")]
    L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    pub day: Day,
    pub block: Block,
    pub week: Option<Week>,
    pub duration: u8,
}

impl From<&Time> for u128 {
    fn from(time: &Time) -> Self {
        Self::MAX & ((1u128 << time.duration) - 1) << time.block.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Type {
    #[strum(serialize = "P", to_string = "Lecture")]
    P,
    #[strum(serialize = "C", to_string = "Tutorial")]
    C,
    #[strum(serialize = "L", to_string = "Lab")]
    L,
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl PartialEq for SubPar<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.0, other.0) && self.1 == other.1
    }
}
