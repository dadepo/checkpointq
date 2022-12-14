use std::collections::{HashMap};
use futures::StreamExt;
use crate::client::{ResponsePayload};


pub fn group_by_root_hash(raw_result: Vec<ResponsePayload>) -> HashMap<String, Vec<ResponsePayload>> {
    let (successes, failures): (Vec<ResponsePayload>, Vec<ResponsePayload>) = raw_result.into_iter().partition(|raw| {
        raw.payload.is_ok()
    });

    let mut hash_to_value_map: HashMap<String, Vec<ResponsePayload>> = HashMap::new();
    successes.into_iter().for_each(|entry| {
        hash_to_value_map
            .entry(entry.payload.as_ref().unwrap().data.finalized.root.clone())
            .or_default()
            .push(entry)
    });

    hash_to_value_map
}