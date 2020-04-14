use llrb_index::Llrb;

use std::ffi;

use crate::{types, core::{Result, Transaction, Durable, Store}};

struct Cache<S> where S: Store {
    db: S,

    commodities: Llrb<String, types::Commodity>,
    companies: Llrb<String, types::Company>,
    ledgers: Llrb<String, types::Ledger>,
    entries: Llrb<String, types::JournalEntry>,
}

impl<S> Store for Cache<S> where S: Store {
    type Txn = CacheTransaction<S>;

    fn create(dir: &ffi::OsStr, w: types::Workspace) -> Result<Self> {
        let db = S::create(dir, w)?;
        let mut c = Cache {
            db,

            commodities: Llrb::new("cache-commodities"),
            companies: Llrb::new("cache-companies"),
            ledgers: Llrb::new("cache-ledgers"),
            entries: Llrb::new("cache-entries"),
        };

        c.load()?;
        Ok(c)
    }

    fn open(dir: &ffi::OsStr) -> Result<Self> {
        let db = S::open(dir)?;
        let mut c = Cache {
            db,

            commodities: Llrb::new("cache-commodities"),
            companies: Llrb::new("cache-companies"),
            ledgers: Llrb::new("cache-ledgers"),
            entries: Llrb::new("cache-entries"),
        };

        c.load()?;
        Ok(c)
    }

    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable
    {
        self.db.put(value)
    }

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable
    {
        self.db.get(key)
    }

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable
    {
        self.db.delete(key)
    }

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable
    {
        self.db.iter()
    }

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>>
    {
        self.db.iter_journal(from, to)
    }

    fn commit(&mut self) -> Result<()> {
        todo!()
    }

    fn pull(&mut self) -> Result<()> {
        todo!()
    }

    fn push(&mut self) -> Result<()> {
        todo!()
    }

    fn begin(self) -> Result<Self::Txn> {
        todo!()
    }
}

impl<S> Cache<S> where S: Store {
    fn load(&mut self) -> Result<()> {
        let commodities = self.iter::<types::Commodity>()?;
        let companies = self.iter::<types::Company>()?;
        let ledgers = self.iter::<types::Ledger>()?;
        let entries = {
            let from = chrono::Utc::now();
            let to: chrono::DateTime<chrono::Utc> = chrono::Local::now().into();
            self.iter_journal(from, to)?
        };

        for c in commodities {
            let c = c?;
            self.commodities.set(c.to_key(), c);
        }
        for c in companies {
            let c = c?;
            self.companies.set(c.to_key(), c);
        }
        for l in ledgers {
            let l = l?;
            self.ledgers.set(l.to_key(), l);
        }
        for e in entries {
            let e = e?;
            self.entries.set(e.to_key(), e);
        }

        Ok(())
    }
}

pub struct CacheTransaction<S> where S: Store {
    db: Cache<S>,
}

impl<S> Transaction<Cache<S>> for CacheTransaction<S> where S: Store {
    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        self.db.put(value)
    }

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        self.db.get(key)
    }

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        self.db.delete(key)
    }

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable,
    {
        self.db.iter()
    }

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>> {
        self.db.iter_journal(from, to)
    }

    fn end(self) -> Result<Cache<S>> {
        Ok(self.db)
    }
}
