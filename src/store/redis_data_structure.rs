use std::collections::HashMap;

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

pub struct SetOptionalArgs {
    expiry: Option<SetExpiry>
}

// Only 1 of 'ex' and 'px' can have a value
struct SetExpiry {
    ex: Option<u8>,   // expire time in seconds
    px: Option<u8>    // expire time in milliseconds
}