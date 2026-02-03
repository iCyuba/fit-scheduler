use std::{collections::HashMap, str::FromStr};

use thiserror::Error;

use crate::{kos, parsed};

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
    Int(#[from] std::num::TryFromIntError),

    #[error(transparent)]
    NumEnum(#[from] num_enum::TryFromPrimitiveError<parsed::Day>),
}

impl TryFrom<kos::parallels::Parallel> for parsed::Parallel {
    type Error = ParallelParseError;

    fn try_from(value: kos::parallels::Parallel) -> Result<Self, Self::Error> {
        if value.timetable.len() != 1 {
            return Err(ParallelParseError::InvalidTimetableLen);
        }

        let timetable = &value.timetable[0];

        Ok(Self {
            time: parsed::Time {
                day: parsed::Day::try_from(u8::try_from(timetable.day_number)?)?,
                block: parsed::Block::from_str(&timetable.ticket_start.0)?,
                biweekly: timetable.even_odd_week.is_some(),
            },
            kind: parsed::Type::from_str(&value.parallel_type.code.0)?,
        })
    }
}
