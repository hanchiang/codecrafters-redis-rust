use std::collections::HashMap;
use chrono::{DateTime, Utc};

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
pub struct LinkedListNode {
    data: String,
    next: Option<Box<LinkedList>>,
}

#[derive(Debug)]
pub struct Set {
    data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Hash {
    data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct DateTimeMeta {
    pub created_at: DateTime<Utc>,
    pub expire_at: Option<DateTime<Utc>>
}

pub struct DateTimeMetaBuilder {
    created_at: DateTime<Utc>,
    expire_at: Option<DateTime<Utc>>
}

impl DateTimeMetaBuilder {
    pub fn new(created_at: DateTime<Utc>) -> DateTimeMetaBuilder {
        DateTimeMetaBuilder {
            created_at,
            expire_at: None
        }
    }

    pub fn expire_at(mut self, expire_at: Option<DateTime<Utc>>) -> DateTimeMetaBuilder {
        self.expire_at = expire_at;
        self
    }

    pub fn build(self) -> DateTimeMeta {
        DateTimeMeta {
            created_at: self.created_at,
            expire_at: self.expire_at
        }
    }
}



