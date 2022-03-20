use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Once;

use serial_test::serial;

use crate::store::redis_data_structure::{DataType, DateTimeMeta, DateTimeMetaBuilder};
use crate::store::redis_operation::SetOptionalArgs;

static mut STORE: Option<RedisStore> = None;
static INIT: Once = Once::new();
static mut INIT_COUNT: u8 = 0;

#[derive(Debug)]
pub struct RedisStore {
    data: HashMap<String, DataType>,
    date_time: HashMap<String, DateTimeMeta>,
}

pub trait Store {
    fn initialise();

    fn get_store() -> &'static mut RedisStore;

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
    #[cfg(not(feature = "integration_test"))]
    fn initialise() {
        INIT.call_once(|| unsafe {
            STORE = Some(RedisStore {
                data: HashMap::new(),
                date_time: HashMap::new(),
            });
            INIT_COUNT += 1;
            println!("Store is initialised.");
        });
    }

    // Using #[cfg(test)] has no effect when running integration tests
    // because they are located in 'test/'
    #[cfg(feature = "integration_test")]
    fn initialise() {
        unsafe {
            STORE = Some(RedisStore {
                data: HashMap::new(),
                date_time: HashMap::new(),
            });
            INIT_COUNT += 1;
        }
        println!("Store is initialised in test mode.");
    }

    fn get_store() -> &'static mut RedisStore {
        unsafe {
            if STORE.is_none() {
                panic!("Store is not initialised.");
            }

            STORE.as_mut().unwrap()
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
        let key_string = String::from(key);
        let key_string_clone = key_string.clone();

        let insert_data_result = self
            .data
            .insert(key_string, DataType::String(String::from(value)));

        let now = Utc::now();
        let mut date_time_meta_builder = DateTimeMetaBuilder::new(now);

        if opt.is_none() {
            self.date_time
                .insert(key_string_clone, date_time_meta_builder.build());
            return insert_data_result;
        }

        let set_args = opt.as_ref().unwrap();
        if set_args.expire_in_ms.is_none() {
            self.date_time
                .insert(key_string_clone, date_time_meta_builder.build());
            return insert_data_result;
        }

        let expire_in_option = set_args.expire_in_ms.as_ref();

        if expire_in_option.is_some() {
            let duration = expire_in_option.unwrap();
            let expire_at = now.checked_add_signed(Duration::milliseconds(*duration as i64));
            date_time_meta_builder = date_time_meta_builder.expire_at(expire_at);
        }

        self.date_time
            .insert(key_string_clone, date_time_meta_builder.build());
        insert_data_result
    }

    fn is_key_expired(&self, key: &str) -> bool {
        let now = Utc::now();
        let date_time_meta_option = self.date_time.get(key);

        if date_time_meta_option.is_none() {
            println!(
                "key {} is not found when checking whether it has expired",
                key
            );
            return false;
        }

        let date_time_meta = date_time_meta_option.unwrap();
        if date_time_meta.expire_at.is_none() {
            return false;
        }

        date_time_meta.expire_at.unwrap() < now
    }

    fn delete(&mut self, keys: Vec<&str>) -> u64 {
        let mut delete_count = 0;

        for key in keys {
            if self.data.remove(key).is_some() {
                self.date_time.remove(key);
                delete_count += 1;
                println!("Key {} is removed.", key);
            }
        }
        delete_count
    }
}

impl RedisStore {
    pub fn reset() {
        unsafe {
            if STORE.is_none() {
                println!("Store is already None.");
                return;
            }

            let store = STORE.as_mut().unwrap();
            store.data = HashMap::new();
            store.date_time = HashMap::new();
            STORE = None;
            INIT_COUNT = 0;

            println!("Store is reset.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;
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
    #[should_panic(expected = "Store is not initialised.")]
    fn panic_if_get_store_before_store_is_initialised() {
        with_reset_redis(|| {
            RedisStore::get_store();
        });
    }

    #[test]
    #[serial]
    fn can_reset_store_correctly() {
        with_reset_redis(|| {
            RedisStore::initialise();

            {
                let store = &mut RedisStore::get_store();
                let key = "key";
                let value = "value";

                let set_result = store.set(key, value, &None);
            }

            {
                let store = RedisStore::get_store();
                let key = "key";
                let result = store.get("key");
                assert_eq!(result.unwrap(), "value");
            }

            RedisStore::reset();
            RedisStore::initialise();

            let store = RedisStore::get_store();
            let keys: Vec<String> = store.data.keys().cloned().collect();
            assert_eq!(keys.len(), 0);
        })
    }

    #[test]
    #[serial]
    fn get_returns_none_if_key_is_not_found() {
        with_reset_redis(|| {
            RedisStore::initialise();

            let store = RedisStore::get_store();
            let result = store.get("random");
            assert!(result.is_none());
        })
    }

    #[test]
    #[serial]
    fn get_returns_string_if_key_is_found_and_date_time_meta_is_set_correctly() {
        // created_at is set, expire_at is not set
        {
            with_reset_redis(|| {
                RedisStore::initialise();

                let store = &mut RedisStore::get_store();
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
            })
        }

        // created_at and expire_at are set
        {
            with_reset_redis(|| {
                RedisStore::initialise();

                let store = &mut RedisStore::get_store();
                let key = "key";
                let value = "value";

                let expire_in = 50;
                let set_args = Some(SetOptionalArgs {
                    expire_in_ms: Some(expire_in),
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
            })
        }
    }

    #[test]
    #[serial]
    fn can_delete_key() {
        with_reset_redis(|| {
            RedisStore::initialise();

            {
                let store = &mut RedisStore::get_store();
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
        })
    }

    #[test]
    #[serial]
    fn key_expires_correctly() {
        with_reset_redis(|| {
            RedisStore::initialise();

            {
                let store = &mut RedisStore::get_store();
                let key = "key";
                let value = "value";

                let expire_in = 50;
                let set_args = Some(SetOptionalArgs {
                    expire_in_ms: Some(expire_in),
                });
                store.set(key, value, &set_args);

                let result = store.get(key);
                assert!(result.is_some());
                assert_eq!(result.unwrap(), value);

                thread::sleep(std::time::Duration::from_millis(expire_in));

                let is_expired = store.is_key_expired(key);
                assert!(is_expired);
            }
        })
    }
}
