use data::kos::{courses::Course, parallels::Parallel};
use tokio::fs::{create_dir_all, write};

use crate::{client::get_client, fetch::fetch_paginated};

pub mod api;
pub mod client;
pub mod fetch;

/// Inspired by:
/// https://github.com/antoninkriz/CTU-TimeTable-Generator/tree/main/kos-loader

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_client()?;

    let courses = fetch_paginated::<Course>(&client).await?;

    create_dir_all("data/cached").await?;

    write(
        "data/cached/courses.json",
        serde_json::to_string_pretty(&courses)?,
    )
    .await?;

    let parallels = fetch_paginated::<Parallel>(&client).await?;

    create_dir_all("data/cached").await?;

    write(
        "data/cached/parallels.json",
        serde_json::to_string_pretty(&parallels)?,
    )
    .await?;

    Ok(())
}
