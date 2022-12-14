use std::collections::{HashMap};
use crate::client::{FailurePayload, GroupedResult, ResponsePayload, SuccessPayload};


pub fn group_success_failure(response_payload: Vec<ResponsePayload>) -> GroupedResult {
    let (successes, failures): (Vec<SuccessPayload>, Vec<FailurePayload>) = response_payload
        .into_iter()
        .fold((vec![], vec![]), |mut acc, result| {
            if let Ok(success) = result.payload {
                acc.0.push(SuccessPayload {
                    payload: success,
                    endpoint: result.endpoint
                })
            } else {
                acc.1.push(FailurePayload {
                    payload: result.payload.err().unwrap(), // guarantee not to panic
                    endpoint: result.endpoint
                });
            }
            acc
    });

    let mut hash_to_successes: HashMap<String, Vec<SuccessPayload>> = HashMap::new();
    successes.into_iter().for_each(|entry| {
        hash_to_successes
            .entry(entry.payload.data.finalized.root.clone())
            .or_default()
            .push(entry)
    });


    GroupedResult {
        success: hash_to_successes,
        failure: failures
    }
}