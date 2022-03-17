use std::borrow::{BorrowMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Once, RwLock};

use lazy_static::lazy_static;
use serial_test::serial;

// TODO: sorted set, bit array, hyperloglog, stream
#[derive(Debug)]
pub enum DataType {
    String(String),
    List(LinkedList),
}

#[derive(Debug)]
pub struct LinkedList {
    head: LinkedListNode,
}

#[derive(Debug)]
struct LinkedListNode {
    data: String,
    next: Option<Box<LinkedList>>,
}

#[derive(Debug)]
pub struct Set<T> {
    data: HashMap<T, String>,
}

#[derive(Debug)]
pub struct Hash {
    data: HashMap<String, String>,
}

// Is this the best way to store data?
lazy_static! {
    static ref STORE: RwLock<Option<RedisStore>> = RwLock::new(None);
}

static INIT: Once = Once::new();
static mut INIT_COUNT: u8 = 0;

#[derive(Debug)]
pub struct RedisStore {
    data: HashMap<String, DataType>,
}

pub trait Store {
    fn initialise();

    fn get_store() -> &'static RwLock<Option<RedisStore>>;

    fn get(&self, key: &str) -> Option<&str>;

    fn set(&mut self, key: &str, value: &str) -> Option<DataType>;
}

impl Store for RedisStore {
    fn initialise() {
        INIT.call_once(|| unsafe {
            *STORE.write().unwrap() = Some(RedisStore {
                data: HashMap::new(),
            });
            INIT_COUNT += 1;
            println!("Store is initialised");
        });
    }

    fn get_store() -> &'static RwLock<Option<RedisStore>> {
        unsafe {
            if INIT_COUNT == 0 {
                panic!("Store is not initialised");
            }
            &STORE
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        let value = self.data.get(key);

        if value.is_none() {
            return None;
        }

        let data_unwrapped = value.unwrap();

        if let DataType::String(string) = data_unwrapped {
            return Some(string);
        }
        None
    }

    fn set(&mut self, key: &str, value: &str) -> Option<DataType> {
        self.data
            .insert(String::from(key), DataType::String(String::from(value)))
    }
}

impl RedisStore {
    pub fn initialise_test() {
        match STORE.try_write() {
            Ok(mut store_lock) => {
                let mut wrapped_store = store_lock.deref_mut();
                if wrapped_store.is_some() {
                    println!("Store is already initialised");
                    return;
                }

                *wrapped_store.deref_mut() = Some(RedisStore {
                    data: HashMap::new(),
                });

                unsafe {
                    INIT_COUNT += 1;
                }
            }
            Err(e) => {
                println!("Error when getting write lock for store: {:?}", e);
                return;
            }
        }
    }

    pub fn reset() {
        match STORE.try_write() {
            Ok(mut store_lock) => {
                let store_lock = store_lock.borrow_mut();
                if store_lock.is_none() {
                    println!("Store is already None.");
                    return;
                }

                let wrapped_store = store_lock.deref_mut();
                if let Some(store) = wrapped_store {
                    store.data = HashMap::new();
                }
                *wrapped_store = None;

                unsafe {
                    INIT_COUNT = 0;
                }
                println!("Store is reset!");
            }
            Err(e) => {
                println!("Unable to get write lock for store: {:?}", e);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::BorrowMut;
    use std::thread;

    fn run_test<F>(test: F)
    where
        F: FnOnce() -> (),
    {
        test();
        RedisStore::reset();
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Store is not initialised")]
    fn panic_if_get_store_before_store_is_initialised() {
        run_test(|| {
            RedisStore::get_store();
        });
    }

    #[test]
    #[serial]
    fn should_initialise_store_only_once() {
        run_test(|| {
            let thread1 = thread::spawn(|| {
                RedisStore::initialise_test();
            });

            let thread2 = thread::spawn(|| {
                RedisStore::initialise_test();
            });

            thread1.join();
            thread2.join();

            unsafe {
                assert_eq!(INIT_COUNT, 1);
            }
        });
    }

    #[test]
    #[serial]
    fn get_returns_none_if_key_is_not_found() {
        run_test(|| {
            RedisStore::initialise_test();
            let store_lock = RedisStore::get_store();
            let store_lock_guard = store_lock.read().unwrap();

            if let Some(store) = store_lock_guard.deref() {
                let result = store.get("random");
                assert!(result.is_none());
            };
        })
    }

    #[test]
    #[serial]
    fn get_returns_string_if_key_is_found() {
        run_test(|| {
            RedisStore::initialise_test();

            let mut store_lock = RedisStore::get_store().write();
            let mut store_guard = store_lock.unwrap();
            let wrapped_store = store_guard.as_mut();
            if wrapped_store.is_some() {
                let mut store = wrapped_store.unwrap();
                let key = "key";
                let value = "value";

                store.set(key, value);

                let result = store.get(key);
                assert!(result.is_some());
                assert_eq!(result.unwrap(), value);
            }
        })
    }

    #[test]
    #[serial]
    fn can_reset_correctly() {
        run_test(|| {
            RedisStore::initialise_test();

            {
                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    store.set(key, value);

                    let result = store.get(key);
                    assert_eq!(result.unwrap(), value);
                }
            }

            RedisStore::reset();
            RedisStore::initialise_test();

            let store_lock = RedisStore::get_store();
            let store_guard = store_lock.read().unwrap();
            if let Some(store) = store_guard.deref() {
                let keys: Vec<String> = store.data.keys().cloned().collect();
                assert_eq!(keys.len(), 0);
            }
        })
    }
}
