struct Commodity {
    name: String,
    value: f64,
    precision: usize,
    factor: u64,
    tags: Vec<Tag>,
    notes: Vec<String>,
}
