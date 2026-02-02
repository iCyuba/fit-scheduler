pub mod kos {
    pub mod courses {
        use typify::import_types;

        import_types!("schema/courses.json");
    }

    pub mod parallels {
        use typify::import_types;

        import_types!("schema/parallels.json");
    }
}
