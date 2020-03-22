struct Creditor {
    account: Ledger,
    commodity: Commodity,
    tags: Vec<Tag>,
    notes: Vec<String>,
}

struct Debitor {
    account: Ledger,
    commodity: Commodity,
    tags: Vec<Tag>,
    notes: Vec<String>,
}

struct Transaction {
    payee: String,
    created: SystemTime,
    creditors: Vec<Creditor>,
    debitors: Vec<Debitor>,
    tags: Vec<Tag>,
    notes: Vec<String>,
    comments: Vec<String>,
}
