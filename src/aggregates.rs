use llrb_index::Llrb;

use crate::{
    core::{Reduce, Result},
    types,
};

struct FrequentlyUsedCompanies {
    companies: Llrb<String, usize>,
}

impl FrequentlyUsedCompanies {
    fn new() -> FrequentlyUsedCompanies {
        FrequentlyUsedCompanies {
            companies: Llrb::new("frequently-used-companies"),
        }
    }

    fn frequent_companies(&self, limit: usize) -> Vec<String> {
        let mut items: Vec<(String, usize)> = self.companies.iter().collect();
        items.sort_by(|x, y| x.1.cmp(&y.1));
        items.into_iter().take(limit).map(|x| x.0).collect()
    }
}

impl Reduce<types::JournalEntry> for FrequentlyUsedCompanies {
    fn reduce(&mut self, doc: &types::JournalEntry) -> Result<()> {
        for c in doc.creditors.iter() {
            let (cname, _): (String, String) = c.ledger.clone().into();
            self.companies
                .set(cname.clone(), self.companies.get(&cname).unwrap_or(1));
        }
        for d in doc.debitors.iter() {
            let (cname, _): (String, String) = d.ledger.clone().into();
            self.companies
                .set(cname.clone(), self.companies.get(&cname).unwrap_or(1));
        }

        Ok(())
    }
}
