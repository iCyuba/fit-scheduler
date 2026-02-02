use data::kos::{courses::Course, parallels::Parallel};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Query {
    pub size: i64,
    pub sort: &'static str,
    pub page: i64,
    pub query: Option<&'static str>,
    pub expanded: Option<&'static str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub elements: Vec<T>,
    pub page: Page,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub page_size: i64,
    pub page_number: i64,
    pub total_pages: i64,
    pub total_elements: i64,
}

pub trait Fetchable {
    fn kos_path() -> &'static str;

    fn query() -> Query {
        Query::default()
    }
}

impl Default for Query {
    fn default() -> Self {
        Self {
            size: 1000,
            sort: "id",
            page: 0,
            query: None,
            expanded: None,
        }
    }
}

impl Fetchable for Course {
    fn kos_path() -> &'static str {
        "courses"
    }
}

impl Fetchable for Parallel {
    fn kos_path() -> &'static str {
        "timetables/parallel-classes"
    }

    fn query() -> Query {
        Query {
            query: Some("semesterId==B252"),
            expanded: Some("timetable.room"),
            ..Default::default()
        }
    }
}
