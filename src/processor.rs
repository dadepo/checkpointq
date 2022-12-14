use std::collections::{HashMap};
use futures::StreamExt;
use crate::client::{FailurePayload, GroupedResult, ResponsePayload, SuccessPayload};


pub fn group_success_failure(responsePayload: Vec<ResponsePayload>) -> GroupedResult {
    let (successes, failures): (Vec<SuccessPayload>, Vec<FailurePayload>) = responsePayload
        .into_iter()
        .fold((vec![], vec![]), |mut acc, result| {
        match result.payload {
            Ok(success) => {
                acc.0.push(SuccessPayload {
                    payload: success,
                    endpoint: result.endpoint
                });
                acc
            },
            Err(err) => {
                acc.1.push(FailurePayload {
                    payload: err,
                    endpoint: result.endpoint
                });
                acc
            }
        }
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