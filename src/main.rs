use anyhow::Context;

use crate::data::{Block, Day, Parallel};

pub mod data;
pub mod json;
pub mod schedule;

static DATA: &str = include_str!("data.json");

const CURR: &str = "B252";

fn main() -> anyhow::Result<()> {
    let mut data = serde_json::from_str::<json::Semesters>(DATA)?;

    let subjects = data
        .0
        .remove(CURR)
        .context("Missing semester")?
        .into_iter()
        .filter(|s| {
            matches!(
                s.code.as_str(),
                "BI-PA2.21" | "BI-MA1.21" | "BI-DBS.21" | "BI-SAP.21" | "BI-LA2.21" | "A0B04KS2"
            )
        });

    let mut subjects: Vec<data::Subject> = Vec::from_iter(subjects);

    for sub in &mut subjects {
        if matches!(sub.code.as_str(), "BI-MA1.21") {
            sub.labs.clear();
            sub.lectures.retain(|p| p.day == Day::Thursday);
        }

        if matches!(sub.code.as_str(), "BI-DBS.21") {
            sub.labs.retain(|p| p.day == Day::Wednesday);
        }

        let filter = |p: &Parallel| match p.day {
            Day::Thursday => matches!(p.block, Block::_12_45 | Block::_14_30 | Block::_16_15),
            Day::Friday => false,
            _ => !matches!(p.block, Block::_7_30 | Block::_18_00),
        };

        sub.lectures.retain(filter);
        sub.seminars.retain(filter);
        sub.labs.retain(filter);

        sub.lectures.dedup();
        sub.seminars.dedup();
        sub.labs.dedup();
    }

    let mut options = 0;
    schedule::Schedule::choose(
        &subjects,
        &mut |(_, par), choices| {
            let mut consec = 0;

            for i in (0..par.block as usize).rev() {
                if choices.0[par.day as usize][i].is_none() {
                    break;
                }

                consec += 1;
            }

            for i in par.block as usize + 1..7 {
                if choices.0[par.day as usize][i].is_none() {
                    break;
                }

                consec += 1;
            }

            consec < 3
        },
        &mut |c| {
            options += 1;

            println!("{options}{c}");
        },
    );

    eprintln!("Total: {options}");

    Ok(())
}
