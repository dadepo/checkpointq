use std::collections::{HashMap};
use futures::StreamExt;
use crate::client::{FailurePayload, GroupedResult, ResponsePayload, SuccessPayload};


pub fn group_success_failure(raw_result: Vec<ResponsePayload>) -> GroupedResult {
    let (successes, failures): (Vec<ResponsePayload>, Vec<ResponsePayload>) = raw_result.into_iter().partition(|raw| {
        raw.payload.is_ok()
    });

    let mut hash_to_value_map: HashMap<String, Vec<SuccessPayload>> = HashMap::new();
    successes.into_iter().for_each(|entry| {
        hash_to_value_map
            .entry(entry.payload.as_ref().unwrap().data.finalized.root.clone())
            .or_default()
            .push(SuccessPayload {
                payload: entry.payload.unwrap(),
                endpoint: entry.endpoint,
            })
    });


    GroupedResult {
        success: hash_to_value_map,
        failure: failures.into_iter().map(|failure| {
            match failure.payload {
                Ok(_) => panic!("Unexpected success processed as failure"),
                Err(e) => FailurePayload {
                    payload: e,
                    endpoint: failure.endpoint
                }
            }
        }).collect()
    }
}