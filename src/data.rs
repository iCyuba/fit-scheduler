use core::fmt;
use std::fmt::Display;

use crate::json;

pub use crate::json::Type;

pub type SubPar<'s> = (&'s Subject, &'s Parallel);

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
    pub day: Day,
    pub block: Block,
    pub kind: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[repr(u8)]
pub enum Block {
    _7_30,
    _9_15,
    _11_00,
    _12_45,
    _14_30,
    _16_15,
    _18_00,
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::_7_30 => write!(f, " 7:30"),
            Block::_9_15 => write!(f, " 9:15"),
            Block::_11_00 => write!(f, "11:00"),
            Block::_12_45 => write!(f, "12:45"),
            Block::_14_30 => write!(f, "14:30"),
            Block::_16_15 => write!(f, "16:15"),
            Block::_18_00 => write!(f, "18:00"),
        }
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Display for Parallel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} @ {}",
            match self.kind {
                Type::C => "C",
                Type::L => "L",
                Type::P => "P",
            },
            self.block
        )
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl TryFrom<u8> for Day {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value.into() {
            0 => Ok(Day::Sunday),
            1 => Ok(Day::Monday),
            2 => Ok(Day::Tuesday),
            3 => Ok(Day::Wednesday),
            4 => Ok(Day::Thursday),
            5 => Ok(Day::Friday),
            6 => Ok(Day::Saturday),
            _ => Err(()),
        }
    }
}

impl FromIterator<json::Subject> for Vec<Subject> {
    fn from_iter<T: IntoIterator<Item = json::Subject>>(iter: T) -> Self {
        iter.into_iter()
            .map(|s| {
                let mut sub = Subject {
                    code: s.code,
                    name: s.name,
                    lectures: vec![],
                    seminars: vec![],
                    labs: vec![],
                };

                for p in &s.parallels {
                    let container = match p.type_ {
                        Type::P => &mut sub.lectures,
                        Type::C => &mut sub.seminars,
                        Type::L => &mut sub.labs,
                    };

                    if p.timetable.is_empty() {
                        continue;
                    }

                    assert_eq!(p.timetable.len(), 1);
                    let timetable = &p.timetable[0];

                    let par = Parallel {
                        // subject: &sub.code,
                        day: Day::try_from(timetable.day as u8).unwrap(),
                        block: match &timetable.start[0..2] {
                            [7, 30] => Block::_7_30,
                            [9, 15] => Block::_9_15,
                            [11, 00] => Block::_11_00,
                            [12, 45] => Block::_12_45,
                            [14, 30] => Block::_14_30,
                            [16, 15] => Block::_16_15,
                            [18, 00] => Block::_18_00,

                            [8, 15] => Block::_7_30,
                            [10, 00] => Block::_9_15,
                            [11, 45] => Block::_11_00,
                            [13, 30] => Block::_12_45,
                            [15, 15] => Block::_14_30,
                            [17, 00] => Block::_16_15,
                            [18, 45] => Block::_18_00,

                            _ => panic!(),
                        },
                        kind: p.type_,
                    };

                    container.push(par);
                }

                sub
            })
            .collect()
    }
}
