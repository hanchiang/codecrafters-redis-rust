use redis_starter_rust::request_response::redis::RedisStore;

pub fn with_reset_redis<F>(test: F)
where
    F: FnOnce() -> (),
{
    test();
    RedisStore::reset();
}
