use anyhow::Context;

use crate::data::{Block, Day, Parallel};

pub mod choices;
pub mod data;
pub mod json;
pub mod scheduler;

static DATA: &str = include_str!("data.json");

const CURR: &str = "B252";

fn main() -> anyhow::Result<()> {
    let mut data = serde_json::from_str::<json::Semesters>(DATA)?;
    let mut subjects: Vec<data::Subject> = data
        .0
        .remove(CURR)
        .context("Missing semester")?
        .into_iter()
        .filter(|s| {
            matches!(
                s.code.as_str(),
                "BI-PA2.21" | "BI-MA1.21" | "BI-DBS.21" | "BI-SAP.21" | "BI-LA2.21" | "A0B04KS2"
            )
        })
        .map(data::Subject::from)
        .collect();

    for sub in &mut subjects {
        if matches!(sub.code.as_str(), "BI-MA1.21") {
            sub.labs.clear();
            sub.lectures.retain(|p| p.time.day == Day::Thursday);
        }

        if matches!(sub.code.as_str(), "BI-DBS.21") {
            sub.labs.retain(|p| p.time.day == Day::Wednesday);
        }

        let filter = |p: &Parallel| match p.time.day {
            Day::Thursday => matches!(p.time.block, Block::_12_45 | Block::_14_30 | Block::_16_15),
            Day::Friday => false,
            _ => !matches!(p.time.block, Block::_7_30 | Block::_18_00),
        };

        sub.lectures.retain(filter);
        sub.seminars.retain(filter);
        sub.labs.retain(filter);

        sub.lectures.dedup();
        sub.seminars.dedup();
        sub.labs.dedup();
    }

    let mut options = 0;
    scheduler::Scheduler::choose(
        &subjects,
        &mut |(_, par), choices| choices.consecutive(par.time) < 3,
        &mut |c| {
            options += 1;

            println!("{options}{c}");
        },
    );

    eprintln!("Total: {options}");

    Ok(())
}
