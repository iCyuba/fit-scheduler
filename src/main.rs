use data::{Block, SubPar, convert, kos};
use std::fs::read_to_string;
use std::str::FromStr;

pub mod choices;
pub mod json;
pub mod scheduler;

fn main() -> anyhow::Result<()> {
    let courses = read_to_string("data/cached/courses.json")?;
    let parallels = read_to_string("data/cached/parallels.json")?;

    let courses: Vec<kos::courses::Course> = serde_json::from_str(&courses)?;
    let parallels: Vec<kos::parallels::Parallel> = serde_json::from_str(&parallels)?;

    let mut subjects = convert::convert(courses, parallels);
    subjects.retain(|s| {
        matches!(
            s.code.as_str(),
            "BI-AAG.21"
                | "BI-AG1.21"
                | "BI-MA2.21"
                | "BI-APS.21"
                | "BI-UKB.21"
                | "BI-IDO.21"
                | "BI-VR1"
        )
    });

    let from = Block::from_str("9:15")?;
    let to = Block::from_str("18:00")?;

    let mut options = 0;
    let cb = scheduler::SchedulerCallbacks {
        filter: &|SubPar(s, p)| {
            p.time
                .iter()
                .all(|t| t.block.offset >= from.offset && t.block.offset < to.offset)
        },
        select: &|SubPar(_, p), choices| {
            p.time
                .iter()
                .all(|&t| choices.consecutive(t) <= 4 * 60 / 15)
        },
        callback: &mut |choices| {
            options += 1;

            println!("{options}:");
            println!("{choices}");
        },
    };

    scheduler::Scheduler::new(cb).schedule(&subjects);

    eprintln!("Total: {options}");

    Ok(())
}
