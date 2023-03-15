use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::Arc,
};

use rusty_leveldb::{WriteBatch, DB};

use crate::shared_math::b_field_element::BFieldElement;

use super::storage_vec::{Index, StorageVec};

pub enum WriteOperation<ParentKey, ParentValue> {
    Write(ParentKey, ParentValue),
    Delete(ParentKey),
}

pub trait DbTable<ParentKey, ParentValue> {
    fn pull_queue(&mut self) -> Vec<WriteOperation<ParentKey, ParentValue>>;
    fn restore_or_new(&mut self);
}

pub trait StorageReader<ParentKey, ParentValue> {
    fn get(&mut self, key: ParentKey) -> Option<ParentValue>;
}

pub enum VecWriteOperation<Index, T> {
    OverWrite((Index, T)),
    Push(T),
    Pop,
}

pub struct DbtVec<ParentKey, ParentValue, Index, T> {
    reader: Arc<RefCell<dyn StorageReader<ParentKey, ParentValue>>>,
    current_length: Option<Index>,
    key_prefix: u8,
    write_queue: VecDeque<VecWriteOperation<Index, T>>,
    cache: HashMap<Index, T>,
    name: String,
}

impl<ParentKey, ParentValue, Index, T> DbtVec<ParentKey, ParentValue, Index, T>
where
    ParentKey: From<(ParentKey, ParentKey)>,
    ParentKey: From<u8>,
    ParentKey: From<Index>,
    Index: From<ParentValue> + From<u64> + Clone,
{
    // Return the key of ParentKey type used to store the length of the vector
    fn get_length_key(key_prefix: u8) -> ParentKey {
        let const_length_key: ParentKey = 0u8.into();
        let key_prefix_key: ParentKey = key_prefix.into();
        (key_prefix_key, const_length_key).into()
    }

    /// Return the length at the last write to disk
    fn persisted_length(&self) -> Option<Index> {
        self.reader
            .as_ref()
            .borrow_mut()
            .get(Self::get_length_key(self.key_prefix))
            .map(|v| v.into())
    }

    /// Return the key of ParentKey type used to store the element at a given index of Index type
    fn get_index_key(&self, index: Index) -> ParentKey {
        let key_prefix_key: ParentKey = self.key_prefix.into();
        let index_key: ParentKey = index.into();
        (key_prefix_key, index_key).into()
    }

    pub fn new(
        reader: Arc<RefCell<dyn StorageReader<ParentKey, ParentValue>>>,
        key_prefix: u8,
        name: &str,
    ) -> Self {
        let length = None;
        let cache = HashMap::new();
        Self {
            key_prefix,
            reader,
            write_queue: VecDeque::default(),
            current_length: length,
            cache,
            name: name.to_string(),
        }
    }
}

impl<ParentKey, ParentValue, T> StorageVec<T>
    for Arc<RefCell<DbtVec<ParentKey, ParentValue, Index, T>>>
where
    ParentKey: From<Index>,
    ParentValue: From<T>,
    T: Clone + From<ParentValue> + Debug,
    ParentKey: From<(ParentKey, ParentKey)>,
    ParentKey: From<u8>,
    Index: From<ParentValue> + From<u64>,
{
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> Index {
        let current_length = self.as_ref().borrow_mut().current_length;
        if let Some(length) = current_length {
            return length;
        }
        let persisted_length = self.as_ref().borrow_mut().persisted_length();
        if let Some(length) = persisted_length {
            length
        } else {
            0
        }
    }

    fn get(&self, index: Index) -> T {
        // Disallow getting values out-of-bounds
        assert!(
            index < self.len(),
            "Out-of-bounds. Got {index} but length was {}. persisted vector name: {}",
            self.len(),
            self.as_ref().borrow_mut().name
        );

        // try cache first
        if self.as_ref().borrow_mut().cache.contains_key(&index) {
            return self
                .as_ref()
                .borrow_mut()
                .cache
                .get(&index)
                .unwrap()
                .clone();
        }

        // then try persistent storage
        let key: ParentKey = self.as_ref().borrow_mut().get_index_key(index);
        let val = self
            .as_ref()
            .borrow_mut()
            .reader
            .as_ref()
            .borrow_mut()
            .get(key)
            .unwrap_or_else(|| {
                panic!(
                    "Element with index {index} does not exist in {}. This should not happen",
                    self.as_ref().borrow_mut().name
                )
            });
        val.into()
    }

    fn set(&mut self, index: Index, value: T) {
        // Disallow setting values out-of-bounds
        assert!(
            index < self.len(),
            "Out-of-bounds. Got {index} but length was {}. persisted vector name: {}",
            self.len(),
            self.as_ref().borrow_mut().name
        );

        let _old_value = self
            .as_ref()
            .borrow_mut()
            .cache
            .insert(index, value.clone());

        // TODO: If `old_value` is Some(*) use it to remove the corresponding
        // element in the `write_queue` to reduce disk IO.

        self.as_ref()
            .borrow_mut()
            .write_queue
            .push_back(VecWriteOperation::OverWrite((index, value)));
    }

    fn pop(&mut self) -> Option<T> {
        // add to write queue
        self.as_ref()
            .borrow_mut()
            .write_queue
            .push_back(VecWriteOperation::Pop);

        // If vector is empty, return None
        if self.len() == 0 {
            return None;
        }

        // Update length
        *self.as_ref().borrow_mut().current_length.as_mut().unwrap() -= 1;

        // try cache first
        let current_length = self.len();
        if self
            .as_ref()
            .borrow_mut()
            .cache
            .contains_key(&current_length)
        {
            self.as_ref().borrow_mut().cache.remove(&current_length)
        } else {
            // then try persistent storage
            let key = self.as_ref().borrow_mut().get_index_key(current_length);
            self.as_ref()
                .borrow_mut()
                .reader
                .as_ref()
                .borrow_mut()
                .get(key)
                .map(|value| value.into())
        }
    }

    fn push(&mut self, value: T) {
        // add to write queue
        self.as_ref()
            .borrow_mut()
            .write_queue
            .push_back(VecWriteOperation::Push(value.clone()));

        // record in cache
        let current_length = self.len();
        let _old_value = self
            .as_ref()
            .borrow_mut()
            .cache
            .insert(current_length, value);

        // TODO: if `old_value` is Some(_) then use it to remove the corresponding
        // element from the `write_queue` to reduce disk operations

        // update length
        self.as_ref().borrow_mut().current_length = Some(current_length + 1);
    }
}

impl<ParentKey, ParentValue, T> DbTable<ParentKey, ParentValue>
    for DbtVec<ParentKey, ParentValue, Index, T>
where
    ParentKey: From<Index>,
    ParentValue: From<T>,
    T: Clone,
    T: From<ParentValue>,
    ParentKey: From<(ParentKey, ParentKey)>,
    ParentKey: From<u8>,
    Index: From<ParentValue>,
    ParentValue: From<Index>,
{
    /// Collect all added elements that have not yet bit persisted
    fn pull_queue(&mut self) -> Vec<WriteOperation<ParentKey, ParentValue>> {
        let maybe_original_length = self.persisted_length();
        // necessary because we need maybe_original_length.is_none() later
        #[allow(clippy::unnecessary_unwrap)]
        let original_length = if maybe_original_length.is_some() {
            maybe_original_length.unwrap()
        } else {
            0
        };
        let mut length = original_length;
        let mut queue = vec![];
        while let Some(write_element) = self.write_queue.pop_front() {
            match write_element {
                VecWriteOperation::OverWrite((i, t)) => {
                    let key = self.get_index_key(i);
                    queue.push(WriteOperation::Write(key, Into::<ParentValue>::into(t)));
                }
                VecWriteOperation::Push(t) => {
                    let key = self.get_index_key(length);
                    length += 1;
                    queue.push(WriteOperation::Write(key, Into::<ParentValue>::into(t)));
                }
                VecWriteOperation::Pop => {
                    let key = self.get_index_key(length - 1);
                    length -= 1;
                    queue.push(WriteOperation::Delete(key));
                }
            };
        }

        if original_length != length || maybe_original_length.is_none() {
            let key = Self::get_length_key(self.key_prefix);
            queue.push(WriteOperation::Write(
                key,
                Into::<ParentValue>::into(length),
            ));
        }

        self.cache.clear();

        queue
    }

    fn restore_or_new(&mut self) {
        if let Some(length) = self
            .reader
            .as_ref()
            .borrow_mut()
            .get(Self::get_length_key(self.key_prefix))
        {
            self.current_length = Some(length.into());
        } else {
            self.current_length = Some(0);
        }
        self.cache.clear();
        self.write_queue.clear();
    }
}

// possible future extension
// pub struct DbtHashMap<Key, Value, K, V> {
//     parent: Arc<RefCell<DbtSchema<Key, Value>>>,
// }

pub trait StorageSingleton<T>
where
    T: Clone,
{
    fn get(&self) -> T;
    fn set(&mut self, t: T);
}

pub struct DbtSingleton<ParentKey, ParentValue, T> {
    current_value: T,
    old_value: T,
    key: ParentKey,
    reader: Arc<RefCell<dyn StorageReader<ParentKey, ParentValue>>>,
}

impl<ParentKey, ParentValue, T> StorageSingleton<T>
    for Arc<RefCell<DbtSingleton<ParentKey, ParentValue, T>>>
where
    T: Clone + From<ParentValue>,
{
    fn get(&self) -> T {
        self.as_ref().borrow().current_value.clone()
    }

    fn set(&mut self, t: T) {
        self.as_ref().borrow_mut().current_value = t;
    }
}

impl<ParentKey, ParentValue, T> DbTable<ParentKey, ParentValue>
    for DbtSingleton<ParentKey, ParentValue, T>
where
    T: Eq + Clone + Default + From<ParentValue>,
    ParentValue: From<T> + Debug,
    ParentKey: Clone,
{
    fn pull_queue(&mut self) -> Vec<WriteOperation<ParentKey, ParentValue>> {
        if self.current_value == self.old_value {
            vec![]
        } else {
            self.old_value = self.current_value.clone();
            vec![WriteOperation::Write(
                self.key.clone(),
                self.current_value.clone().into(),
            )]
        }
    }

    fn restore_or_new(&mut self) {
        self.current_value = match self.reader.as_ref().borrow_mut().get(self.key.clone()) {
            Some(value) => value.into(),
            None => T::default(),
        }
    }
}

pub struct DbtSchema<ParentKey, ParentValue, Reader: StorageReader<ParentKey, ParentValue>> {
    pub tables: Vec<Arc<RefCell<dyn DbTable<ParentKey, ParentValue>>>>,
    pub reader: Arc<RefCell<Reader>>,
}

impl<ParentKey, ParentValue, Reader: StorageReader<ParentKey, ParentValue> + 'static>
    DbtSchema<ParentKey, ParentValue, Reader>
{
    pub fn new_vec<Index, T>(
        &mut self,
        name: &str,
    ) -> Arc<RefCell<DbtVec<ParentKey, ParentValue, Index, T>>>
    where
        ParentKey: From<Index> + 'static,
        ParentValue: From<T> + 'static,
        T: Clone + From<ParentValue> + 'static,
        ParentKey: From<(ParentKey, ParentKey)>,
        ParentKey: From<u8>,
        Index: From<ParentValue>,
        ParentValue: From<Index>,
        Index: From<u64> + 'static,
        DbtVec<ParentKey, ParentValue, Index, T>: DbTable<ParentKey, ParentValue>,
    {
        assert!(self.tables.len() < 255);
        let reader = self.reader.clone();
        let vector = DbtVec::<ParentKey, ParentValue, Index, T> {
            reader,
            current_length: None,
            key_prefix: self.tables.len() as u8,
            write_queue: VecDeque::new(),
            cache: HashMap::new(),
            name: name.to_string(),
        };
        let arc_refcell_vector = Arc::new(RefCell::new(vector));
        self.tables.push(arc_refcell_vector.clone());
        arc_refcell_vector
    }

    // possible future extension
    // fn new_hashmap<K, V>(&self) -> Arc<RefCell<DbtHashMap<K, V>>> { }

    pub fn new_singleton<S>(
        &mut self,
        key: ParentKey,
    ) -> Arc<RefCell<DbtSingleton<ParentKey, ParentValue, S>>>
    where
        S: Default + Eq + Clone + 'static,
        ParentKey: 'static,
        ParentValue: From<S> + 'static,
        ParentKey: From<(ParentKey, ParentKey)> + From<u8>,
        DbtSingleton<ParentKey, ParentValue, S>: DbTable<ParentKey, ParentValue>,
    {
        let reader = self.reader.clone();
        let singleton = DbtSingleton::<ParentKey, ParentValue, S> {
            current_value: S::default(),
            old_value: S::default(),
            key,
            reader,
        };
        let arc_refcell_singleton = Arc::new(RefCell::new(singleton));
        self.tables.push(arc_refcell_singleton.clone());
        arc_refcell_singleton
    }
}

pub trait StorageWriter<ParentKey, ParentValue> {
    fn persist(&mut self);
    fn restore_or_new(&mut self);
}

#[derive(Clone, PartialEq, Eq)]
pub struct RustyKey(pub Vec<u8>);
impl From<u8> for RustyKey {
    fn from(value: u8) -> Self {
        Self([value].to_vec())
    }
}
impl From<(RustyKey, RustyKey)> for RustyKey {
    fn from(value: (RustyKey, RustyKey)) -> Self {
        let v0 = value.0 .0;
        let v1 = value.1 .0;
        RustyKey([v0, v1].concat())
    }
}
impl From<u64> for RustyKey {
    fn from(value: u64) -> Self {
        RustyKey(value.to_be_bytes().to_vec())
    }
}

#[derive(Debug)]
pub struct RustyValue(Vec<u8>);

impl From<RustyValue> for u64 {
    fn from(value: RustyValue) -> Self {
        u64::from_be_bytes(value.0.try_into().unwrap())
    }
}
impl From<u64> for RustyValue {
    fn from(value: u64) -> Self {
        RustyValue(value.to_be_bytes().to_vec())
    }
}
impl From<RustyValue> for crate::shared_math::tip5::Digest {
    fn from(value: RustyValue) -> Self {
        crate::shared_math::tip5::Digest::new(
            value
                .0
                .chunks(8)
                .map(|ch| {
                    u64::from_be_bytes(ch.try_into().expect("Cannot cast RustyValue into Digest"))
                })
                .map(BFieldElement::new)
                .collect::<Vec<_>>()
                .try_into().expect("Can cast RustyValue into BFieldElements but number does not match that of Digest."),
        )
    }
}
impl From<crate::shared_math::tip5::Digest> for RustyValue {
    fn from(value: crate::shared_math::tip5::Digest) -> Self {
        RustyValue(
            value
                .values()
                .map(|b| b.value())
                .map(u64::to_be_bytes)
                .concat(),
        )
    }
}

/// Database schema and tables logic for RustyLevelDB. You probably
/// want to implement your own storage class after this example so
/// that you can hardcode the schema in new(). But it is nevertheless
/// possible to use this struct and add to the scheme after calling
/// new() (that's what the tests do).
pub struct SimpleRustyStorage {
    db: Arc<RefCell<DB>>,
    schema: DbtSchema<RustyKey, RustyValue, SimpleRustyReader>,
}

impl StorageWriter<RustyKey, RustyValue> for SimpleRustyStorage {
    fn persist(&mut self) {
        let mut write_batch = WriteBatch::new();
        for table in &self.schema.tables {
            let operations = table.as_ref().borrow_mut().pull_queue();
            for op in operations {
                match op {
                    WriteOperation::Write(key, value) => write_batch.put(&key.0, &value.0),
                    WriteOperation::Delete(key) => write_batch.delete(&key.0),
                }
            }
        }

        self.db
            .as_ref()
            .borrow_mut()
            .write(write_batch, true)
            .expect("Could not persist to database.");
    }

    fn restore_or_new(&mut self) {
        for table in &self.schema.tables {
            table.as_ref().borrow_mut().restore_or_new();
        }
    }
}

impl SimpleRustyStorage {
    pub fn new(db: DB) -> Self {
        let db_pointer = Arc::new(RefCell::new(db));
        let reader = SimpleRustyReader {
            db: db_pointer.clone(),
        };
        let schema = DbtSchema::<RustyKey, RustyValue, SimpleRustyReader> {
            tables: Vec::new(),
            reader: Arc::new(RefCell::new(reader)),
        };
        Self {
            db: db_pointer,
            schema,
        }
    }

    pub fn close(&mut self) {
        self.db
            .as_ref()
            .borrow_mut()
            .close()
            .expect("Could not close database.");
    }
}

struct SimpleRustyReader {
    db: Arc<RefCell<DB>>,
}

impl StorageReader<RustyKey, RustyValue> for SimpleRustyReader {
    fn get(&mut self, key: RustyKey) -> Option<RustyValue> {
        self.db.as_ref().borrow_mut().get(&key.0).map(RustyValue)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Default, PartialEq, Eq, Clone, Debug)]
    struct S(Vec<u8>);
    impl From<Vec<u8>> for S {
        fn from(value: Vec<u8>) -> Self {
            S(value)
        }
    }
    impl From<S> for Vec<u8> {
        fn from(value: S) -> Self {
            value.0
        }
    }
    impl From<(S, S)> for S {
        fn from(value: (S, S)) -> Self {
            let vector0: Vec<u8> = value.0.into();
            let vector1: Vec<u8> = value.1.into();
            S([vector0, vector1].concat())
        }
    }
    impl From<RustyValue> for S {
        fn from(value: RustyValue) -> Self {
            Self(value.0)
        }
    }
    impl From<S> for RustyValue {
        fn from(value: S) -> Self {
            Self(value.0)
        }
    }
    impl From<S> for u64 {
        fn from(value: S) -> Self {
            u64::from_be_bytes(value.0.try_into().unwrap())
        }
    }

    #[test]
    fn test_simple_singleton() {
        let singleton_value = S([1u8, 3u8, 3u8, 7u8].to_vec());
        // let opt = rusty_leveldb::Options::default();
        let opt = rusty_leveldb::in_memory();
        let db = DB::open("test-database", opt.clone()).unwrap();

        let mut rusty_storage = SimpleRustyStorage::new(db);
        let mut singleton = rusty_storage
            .schema
            .new_singleton::<S>(RustyKey([1u8; 1].to_vec()));

        // initialize
        rusty_storage.restore_or_new();

        // test
        assert_eq!(singleton.get(), S([].to_vec()));

        // set
        singleton.set(singleton_value.clone());

        // test
        assert_eq!(singleton.get(), singleton_value);

        // persist
        rusty_storage.persist();

        // test
        assert_eq!(singleton.get(), singleton_value);

        // drop
        rusty_storage.close();

        // restore
        let new_db = DB::open("test-database", opt).unwrap();
        let mut new_rusty_storage = SimpleRustyStorage::new(new_db);
        let new_singleton = new_rusty_storage
            .schema
            .new_singleton::<S>(RustyKey([1u8; 1].to_vec()));
        new_rusty_storage.restore_or_new();

        // test
        assert_eq!(new_singleton.get(), singleton_value);
    }

    #[test]
    fn test_simple_vector() {
        let opt = rusty_leveldb::in_memory();
        let db = DB::open("test-database", opt.clone()).unwrap();

        let mut rusty_storage = SimpleRustyStorage::new(db);
        let mut vector = rusty_storage.schema.new_vec::<u64, S>("test-vector");

        // initialize
        rusty_storage.restore_or_new();

        // populate
        vector.push(S([1u8].to_vec()));
        vector.push(S([3u8].to_vec()));
        vector.push(S([4u8].to_vec()));
        vector.push(S([7u8].to_vec()));
        vector.push(S([8u8].to_vec()));

        // test
        assert_eq!(vector.get(0), S([1u8].to_vec()));
        assert_eq!(vector.get(1), S([3u8].to_vec()));
        assert_eq!(vector.get(2), S([4u8].to_vec()));
        assert_eq!(vector.get(3), S([7u8].to_vec()));
        assert_eq!(vector.get(4), S([8u8].to_vec()));
        assert_eq!(vector.len(), 5);

        // persist
        rusty_storage.persist();

        // modify
        let last = vector.pop().unwrap();

        // test
        assert_eq!(last, S([8u8].to_vec()));

        // drop without persisting
        rusty_storage.close();

        // create new database
        let new_db = DB::open("test-database", opt).unwrap();
        let mut new_rusty_storage = SimpleRustyStorage::new(new_db);
        let mut new_vector = new_rusty_storage.schema.new_vec::<u64, S>("test-vector");

        // initialize
        new_rusty_storage.restore_or_new();

        // modify
        new_vector.set(2, S([3u8].to_vec()));
        new_vector.pop();

        // test
        assert_eq!(new_vector.get(0), S([1u8].to_vec()));
        assert_eq!(new_vector.get(1), S([3u8].to_vec()));
        assert_eq!(new_vector.get(2), S([3u8].to_vec()));
        assert_eq!(new_vector.get(3), S([7u8].to_vec()));
        assert_eq!(new_vector.len(), 4);
    }

    #[test]
    fn test_two_vectors_and_singleton() {
        let singleton_value = S([3u8, 3u8, 3u8, 1u8].to_vec());
        let opt = rusty_leveldb::in_memory();
        let db = DB::open("test-database", opt.clone()).unwrap();

        let mut rusty_storage = SimpleRustyStorage::new(db);
        let mut vector1 = rusty_storage.schema.new_vec::<u64, S>("test-vector1");
        let mut vector2 = rusty_storage.schema.new_vec::<u64, S>("test-vector2");
        let mut singleton = rusty_storage
            .schema
            .new_singleton::<S>(RustyKey([1u8; 1].to_vec()));

        // initialize
        rusty_storage.restore_or_new();

        // populate 1
        vector1.push(S([1u8].to_vec()));
        vector1.push(S([3u8].to_vec()));
        vector1.push(S([4u8].to_vec()));
        vector1.push(S([7u8].to_vec()));
        vector1.push(S([8u8].to_vec()));

        // populate 2
        vector2.push(S([1u8].to_vec()));
        vector2.push(S([3u8].to_vec()));
        vector2.push(S([3u8].to_vec()));
        vector2.push(S([7u8].to_vec()));

        // set singleton
        singleton.set(singleton_value.clone());

        // modify 1
        vector1.set(0, S([8u8].to_vec()));

        // test
        assert_eq!(vector1.get(0), S([8u8].to_vec()));
        assert_eq!(vector1.get(1), S([3u8].to_vec()));
        assert_eq!(vector1.get(2), S([4u8].to_vec()));
        assert_eq!(vector1.get(3), S([7u8].to_vec()));
        assert_eq!(vector1.get(4), S([8u8].to_vec()));
        assert_eq!(vector1.len(), 5);
        assert_eq!(vector2.get(0), S([1u8].to_vec()));
        assert_eq!(vector2.get(1), S([3u8].to_vec()));
        assert_eq!(vector2.get(2), S([3u8].to_vec()));
        assert_eq!(vector2.get(3), S([7u8].to_vec()));
        assert_eq!(vector2.len(), 4);
        assert_eq!(singleton.get(), singleton_value);

        // persist and drop
        rusty_storage.persist();
        rusty_storage.close();

        // restore from disk
        let new_db = DB::open("test-database", opt).unwrap();
        let mut new_rusty_storage = SimpleRustyStorage::new(new_db);
        let new_vector1 = new_rusty_storage.schema.new_vec::<u64, S>("test-vector1");
        let new_vector2 = new_rusty_storage.schema.new_vec::<u64, S>("test-vector2");
        new_rusty_storage.restore_or_new();

        // test again
        assert_eq!(new_vector1.get(0), S([8u8].to_vec()));
        assert_eq!(new_vector1.get(1), S([3u8].to_vec()));
        assert_eq!(new_vector1.get(2), S([4u8].to_vec()));
        assert_eq!(new_vector1.get(3), S([7u8].to_vec()));
        assert_eq!(new_vector1.get(4), S([8u8].to_vec()));
        assert_eq!(new_vector1.len(), 5);
        assert_eq!(new_vector2.get(0), S([1u8].to_vec()));
        assert_eq!(new_vector2.get(1), S([3u8].to_vec()));
        assert_eq!(new_vector2.get(2), S([3u8].to_vec()));
        assert_eq!(new_vector2.get(3), S([7u8].to_vec()));
        assert_eq!(new_vector2.len(), 4);
        assert_eq!(singleton.get(), singleton_value);
    }
}
