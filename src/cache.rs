use crate::core::Store;

struct Cache<S> where S: Store {
    db: S

    commodities: Llrb<String, types::Commodity>,
    companies: Llrb<String, types::Company>,
    ledgers: Llrb<String, types::Ledger>,
    entries: Llrb<String, types::JournalEntry>,
}

impl<S> Store for Cache where S: Store {
    type Txn = Transaction<Self>;

    fn create(dir: &ffi::OsStr, w: types::Workspace) -> Result<Self> {
        let db = S::create(dir, w)?;
        Ok(Cache {
            db,

            commodities: Default::default(),
            companies: Default::default(),
            ledgers: Default::default(),
            entries: Default::default(),
        })
    }

    fn open(dir: &ffi::OsStr) -> Result<Self> {
        let db = S::open(dir)?;
        Ok(Cache {
            db,

            commodities: Default::default(),
            companies: Default::default(),
            ledgers: Default::default(),
            entries: Default::default(),
        })
    }

    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable {
        todo!()
        }

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable {
        todo!()
        }

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable {
        todo!()
        }

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable {
        todo!()
        }

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>> {
    todo!()
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
