use anyhow::Context;

use crate::data::{Block, Day, Type};

pub mod choices;
pub mod data;
pub mod json;
pub mod scheduler;

static DATA: &str = include_str!("data.json");

const CURR: &str = "B252";

fn main() -> anyhow::Result<()> {
    let mut data = serde_json::from_str::<json::Semesters>(DATA)?;
    let subjects: Vec<data::Subject> = data
        .0
        .remove(CURR)
        .context("Missing semester")?
        .into_iter()
        .filter(|s| {
            matches!(
                s.code.as_str(),
                "BI-PA2.21"
                    | "BI-MA1.21"
                    | "BI-DBS.21"
                    | "BI-SAP.21"
                    | "BI-LA2.21"
                    | "BI-PSI.21"
                    | "A0B04KS2"
            )
        })
        .map(data::Subject::from)
        .collect();

    let mut options = 0;
    let cb = scheduler::SchedulerCallbacks {
        filter: &|(s, p)| {
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
        select: &|(_, p), choices| choices.consecutive(p.time) < 3,
        callback: &mut |choices| {
            options += 1;

            println!("{options}{choices}");
        },
    };

    scheduler::Scheduler::new(cb).schedule(&subjects);

    eprintln!("Total: {options}");

    Ok(())
}
