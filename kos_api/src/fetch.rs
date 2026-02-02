use std::time::Duration;

use rand::{Rng, rng};
use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::time::sleep;

use crate::{
    api::{Fetchable, Paginated},
    client::KOS,
};

pub async fn fetch_paginated<T>(client: &Client) -> anyhow::Result<Vec<T>>
where
    T: Fetchable + DeserializeOwned,
{
    let url = KOS.join(T::kos_path())?;
    let mut rng = rng();

    let mut vec = vec![];
    let mut query = T::query();

    loop {
        let req = client.get(url.clone()).query(&query).build()?;
        let res = client.execute(req).await?;

        dbg!(&res);

        let curr: Paginated<T> = res.json().await?;

        vec.extend(curr.elements);

        sleep(Duration::from_millis(rng.random_range(1500..2500))).await;

        query.page += 1;

        if query.page >= curr.page.total_pages {
            break;
        }
    }

    Ok(vec)
}
