struct Company {
    name: String,
    aliases: Vec<String>,
    created: SystemTime,
    tags: Vec<Tag>,
    notes: Vec<String>,
    comments: Vec<String>,
}
