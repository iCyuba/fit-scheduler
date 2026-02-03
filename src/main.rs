use std::fs::read_to_string;

use data::{Block, Day, SubPar, Type, convert, kos};

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
            "BI-PA2.21"
                | "BI-MA1.21"
                | "BI-DBS.21"
                | "BI-SAP.21"
                | "BI-LA2.21"
                | "A0B04KS2"
                | "BI-CS1"
        )
    });

    let mut options = 0;
    let cb = scheduler::SchedulerCallbacks {
        filter: &|SubPar(s, p)| {
            (match s.code.as_str() {
                "BI-MA1.21" => match p.kind {
                    Type::L => false,
                    Type::C => true,
                    Type::P => p.time.day == Day::Thursday,
                },

                "BI-DBS.21" => p.kind != Type::L || p.time.day == Day::Wednesday,

                _ => true,
            }) && match p.time.day {
                Day::Thursday => {
                    matches!(p.time.block, Block::_12_45 | Block::_14_30 | Block::_16_15)
                }
                Day::Friday => false,
                _ => !matches!(p.time.block, Block::_7_30 | Block::_18_00),
            }
        },
        select: &|SubPar(_, p), choices| choices.consecutive(p.time) < 3,
        callback: &mut |choices| {
            options += 1;

            println!("{options}{choices}");
        },
    };

    scheduler::Scheduler::new(cb).schedule(&subjects);

    eprintln!("Total: {options}");

    Ok(())
}
