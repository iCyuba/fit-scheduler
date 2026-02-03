use std::{
    fmt::{Debug, Display},
    ptr,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{Display, EnumIter, EnumString};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parallel {
    pub time: Time,
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
    EnumString,
    Display,
)]
#[repr(u8)]
pub enum Block {
    #[strum(to_string = "07:30", serialize = "08:15")]
    _7_30,
    #[strum(to_string = "09:15", serialize = "10:00")]
    _9_15,
    #[strum(to_string = "11:00", serialize = "11:45")]
    _11_00,
    #[strum(to_string = "12:45", serialize = "13:30")]
    _12_45,
    #[strum(to_string = "14:30", serialize = "15:15")]
    _14_30,
    #[strum(to_string = "16:15", serialize = "17:00")]
    _16_15,
    #[strum(to_string = "18:00", serialize = "18:45")]
    _18_00,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    pub day: Day,
    pub block: Block,
    pub biweekly: bool,
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

impl Display for Parallel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.kind, self.time.block)
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl Display for SubPar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl PartialEq for SubPar<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.0, other.0) && self.1 == other.1
    }
}
