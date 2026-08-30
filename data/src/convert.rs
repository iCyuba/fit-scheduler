use smallvec::SmallVec;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

use crate::{BlockParseError, kos, parsed};

pub fn convert(
    courses: impl IntoIterator<Item = kos::courses::Course>,
    parallels: impl IntoIterator<Item = kos::parallels::Parallel>,
) -> Vec<parsed::Subject> {
    let mut subjects: HashMap<_, _> = courses
        .into_iter()
        .map(|c| (c.id, parsed::Subject::from(c)))
        .collect();

    for p in parallels {
        let Some(sub) = subjects.get_mut(&p.course_view.id) else {
            continue;
        };

        let Ok(p) = parsed::Parallel::try_from(p) else {
            continue;
        };

        let container = match p.kind {
            parsed::Type::P => &mut sub.lectures,
            parsed::Type::C => &mut sub.seminars,
            parsed::Type::L => &mut sub.labs,
        };

        container.push(p);
    }

    subjects.into_values().collect()
}

impl From<kos::courses::Course> for parsed::Subject {
    fn from(value: kos::courses::Course) -> Self {
        Self {
            code: value.code,
            name: value.name_cs,
            lectures: vec![],
            seminars: vec![],
            labs: vec![],
        }
    }
}

#[derive(Debug, Error)]
pub enum ParallelParseError {
    #[error("Missing or ambiguous timetable")]
    InvalidTimetableLen,

    #[error(transparent)]
    Strum(#[from] strum::ParseError),

    #[error(transparent)]
    BlockParse(#[from] BlockParseError),

    #[error(transparent)]
    Int(#[from] std::num::TryFromIntError),

    #[error(transparent)]
    NumEnum(#[from] num_enum::TryFromPrimitiveError<parsed::Day>),
}

impl TryFrom<kos::parallels::Parallel> for parsed::Parallel {
    type Error = ParallelParseError;

    fn try_from(value: kos::parallels::Parallel) -> Result<Self, Self::Error> {
        if value.timetable.is_empty() {
            return Err(ParallelParseError::InvalidTimetableLen);
        }

        let time = value
            .timetable
            .iter()
            .map(|v| {
                let start = parsed::Block::from_str(&v.ticket_start.0)?;
                let end = parsed::Block::from_str(&v.ticket_end.0)?;

                let day = parsed::Day::try_from(u8::try_from(v.day_number)?)?;
                let week = v
                    .even_odd_week
                    .as_ref()
                    .map(|w| parsed::Week::from_str(&w.0))
                    .transpose()?;

                Ok(parsed::Time {
                    day,
                    block: start,
                    week,
                    duration: end.offset - start.offset,
                })
            })
            .collect::<Result<SmallVec<_>, Self::Error>>()?;

        Ok(Self {
            time,
            kind: parsed::Type::from_str(&value.parallel_type.code.0)?,
        })
    }
}
