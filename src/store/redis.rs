use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::ops::{Add, Deref, DerefMut};
use std::sync::{Once, RwLock};
use chrono::{DateTime, Duration, Utc};

use lazy_static::lazy_static;
use serial_test::serial;

use crate::store::redis_data_structure::{DataType, DateTimeMeta, DateTimeMetaBuilder};
use crate::store::redis_operation::SetOptionalArgs;

// Is this the best way to store data? Is mutex better?
// https://stackoverflow.com/questions/50704279/when-or-why-should-i-use-a-mutex-over-an-rwlock
lazy_static! {
    static ref STORE: RwLock<Option<RedisStore>> = RwLock::new(None);
}

static INIT: Once = Once::new();
static mut INIT_COUNT: u8 = 0;

#[derive(Debug)]
pub struct RedisStore {
    data: HashMap<String, DataType>,
    date_time: HashMap<String, DateTimeMeta>,
}

pub trait Store {
    fn initialise();

    fn get_store() -> &'static RwLock<Option<RedisStore>>;

    // https://redis.io/commands/get
    fn get(&self, key: &str) -> Option<&str>;

    // https://redis.io/commands/set
    /// Returns None if key is not present previously, or the old value of the key
    fn set(&mut self, key: &str, value: &str, opt: &Option<SetOptionalArgs>) -> Option<DataType>;

    fn is_key_expired(&self, key: &str) -> bool;

    /// Returns number of key that are deleted
    fn delete(&mut self, keys: Vec<&str>) -> u64;
}

impl Store for RedisStore {
    fn initialise() {
        INIT.call_once(|| unsafe {
            *STORE.write().unwrap() = Some(RedisStore {
                data: HashMap::new(),
                date_time: HashMap::new(),
            });
            INIT_COUNT += 1;
            println!("Store is initialised");
        });
    }

    fn get_store() -> &'static RwLock<Option<RedisStore>> {
        unsafe {
            if INIT_COUNT == 0 {
                panic!("Cannot get store because it is not initialised");
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

    fn set(&mut self, key: &str, value: &str, opt: &Option<SetOptionalArgs>) -> Option<DataType> {
        println!("SET: key {}, value: {}, opt: {:?}", key, value, opt);
        let key_string = String::from(key);
        let key_string_clone = key_string.clone();

        let insert_data_result = self.data
            .insert(key_string, DataType::String(String::from(value)));

        let now = Utc::now();
        let mut date_time_meta_builder = DateTimeMetaBuilder::new(now);

        if opt.is_none() {
            self.date_time.insert(key_string_clone, date_time_meta_builder.build());
            return insert_data_result;
        }

        let set_args = opt.as_ref().unwrap();
        if set_args.expire_in_ms.is_none() {
            self.date_time.insert(key_string_clone, date_time_meta_builder.build());
            return insert_data_result;
        }

        let expire_in_option = set_args.expire_in_ms.as_ref();

        if expire_in_option.is_some() {
            let duration = expire_in_option.unwrap();
            let expire_at = now.checked_add_signed(Duration::milliseconds(*duration as i64));
            date_time_meta_builder = date_time_meta_builder.expire_at(expire_at);
        }

        self.date_time.insert(key_string_clone, date_time_meta_builder.build());
        insert_data_result
    }

    fn is_key_expired(&self, key: &str) -> bool {
        let now = Utc::now();
        let date_time_meta_option = self.date_time.get(key);

        if date_time_meta_option.is_none() {
            println!("key {} is not found when checking whether it has expired", key);
            return false;
        }

        let date_time_meta = date_time_meta_option.unwrap();
        if date_time_meta.expire_at.is_none() {
            return false;
        }

        println!("date_time_meta.expire_at: {:?}, now: {:?}", date_time_meta.expire_at.unwrap(), now);
        date_time_meta.expire_at.unwrap() < now
    }

    fn delete(&mut self, keys: Vec<&str>) -> u64 {
        let mut delete_count = 0;

        for key in keys {
            if self.data.remove(key).is_some() {
                self.date_time.remove(key);
                delete_count += 1;
                println!("Remove key {}", key);
            }
        }
        delete_count
    }
}

impl RedisStore {
    // Rename this to initialise, move to Store trait, and use #[cfg(not(test))] macro
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
                    date_time: HashMap::new(),
                });
                println!("Store is initialised.");

                unsafe {
                    INIT_COUNT += 1;
                }
            }
            Err(e) => {
                println!(
                    "Error when getting write lock for store during initialisation: {:?}",
                    e
                );
                return;
            }
        }
    }

    // Rename this to reset, move to Store trait, and use #[cfg(not(test))] macro
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
                    store.date_time = HashMap::new();
                }
                *wrapped_store = None;

                unsafe {
                    INIT_COUNT = 0;
                }
                println!("Store is reset.");
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

    fn with_reset_redis<F>(test: F)
    where
        F: FnOnce() -> (),
    {
        test();
        RedisStore::reset();
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Cannot get store because it is not initialised")]
    fn panic_if_get_store_before_store_is_initialised() {
        with_reset_redis(|| {
            RedisStore::get_store();
        });
    }

    #[test]
    #[serial]
    fn should_initialise_store_only_once() {
        with_reset_redis(|| {
            let thread1 = thread::spawn(|| {
                RedisStore::initialise_test();
            });

            let thread2 = thread::spawn(|| {
                RedisStore::initialise_test();
            });

            thread1.join();
            thread2.join();

            let store_lock = RedisStore::get_store().read().unwrap();
            if let Some(store) = store_lock.deref() {
                let keys: Vec<String> = store.data.keys().cloned().collect();
                assert_eq!(keys.len(), 0);
            } else {
                panic!("Store is empty")
            }

            unsafe {
                assert_eq!(INIT_COUNT, 1);
            }
        });
    }

    #[test]
    #[serial]
    fn can_reset_store_correctly() {
        with_reset_redis(|| {
            RedisStore::initialise_test();

            {
                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    let set_result = store.set(key, value, &None);
                }
            }

            {
                let store_lock = RedisStore::get_store();
                let store_guard = store_lock.read().unwrap();
                if let Some(store) = store_guard.deref() {
                    let result = store.get("key");
                    assert_eq!(result.unwrap(), "value");
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

    #[test]
    #[serial]
    fn get_returns_none_if_key_is_not_found() {
        with_reset_redis(|| {
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
    fn get_returns_string_if_key_is_found_and_date_time_meta_is_set_correctly() {
        // created_at is set, expire_at is not set
        {
            with_reset_redis(|| {
                RedisStore::initialise_test();

                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    store.set(key, value, &None);

                    let result = store.get(key);
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), value);

                    let date_time_meta = store.date_time.get(key);
                    assert!(date_time_meta.is_some());
                    let date_time = date_time_meta.unwrap();
                    assert!(date_time.created_at < Utc::now());
                    assert!(date_time.expire_at.is_none());
                }
            })
        }

        // created_at and expire_at are not set
        {
            with_reset_redis(|| {
                RedisStore::initialise_test();

                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    let expire_in = 50;
                    let set_args = Some(SetOptionalArgs {
                        expire_in_ms: Some(expire_in)
                    });
                    store.set(key, value, &set_args);

                    let result = store.get(key);
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), value);

                    let date_time_meta = store.date_time.get(key);
                    assert!(date_time_meta.is_some());
                    let date_time = date_time_meta.unwrap();
                    assert!(date_time.created_at < Utc::now());
                    assert!(date_time.expire_at.unwrap() > Utc::now());
                }
            })
        }
    }

    #[test]
    #[serial]
    fn can_delete_key() {
        with_reset_redis(|| {
            RedisStore::initialise_test();

            {
                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    store.set(key, value, &None);

                    let result = store.get(key);
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), value);

                    store.delete(vec![key]);

                    let result = store.get(key);
                    assert!(result.is_none());
                }
            }
        })
    }

    #[test]
    #[serial]
    fn key_expires_correctly() {
        with_reset_redis(|| {
            RedisStore::initialise_test();

            {
                let mut store_lock = RedisStore::get_store().write();
                let mut store_guard = store_lock.unwrap();
                let wrapped_store = store_guard.as_mut();
                if wrapped_store.is_some() {
                    let mut store = wrapped_store.unwrap();
                    let key = "key";
                    let value = "value";

                    let expire_in = 50;
                    let set_args = Some(SetOptionalArgs {
                        expire_in_ms: Some(expire_in)
                    });
                    store.set(key, value, &set_args);

                    let result = store.get(key);
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), value);

                    thread::sleep(std::time::Duration::from_millis(expire_in));

                    let is_expired = store.is_key_expired(key);
                    assert!(is_expired);
                }
            }
        })
    }
}
