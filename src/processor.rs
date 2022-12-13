use std::collections::{BTreeMap, HashMap};
use futures::StreamExt;
use crate::client::{Finality_Results, UrlFinalityCheckpointResp};


pub fn group_by_root_hash(raw_result: Finality_Results) -> HashMap<String, Vec<UrlFinalityCheckpointResp>> {
    let (successes, failures): (Finality_Results, Finality_Results) = raw_result.into_iter().partition(|raw| {
        raw.0.is_ok()
    });

    let mut hash_to_value_map: HashMap<String, Vec<UrlFinalityCheckpointResp>> = HashMap::new();
    successes.into_iter().for_each(|entry| {
        hash_to_value_map
            .entry(entry.0.as_ref().unwrap().data.finalized.root.clone())
            .or_default()
            .push({
                let entry_0 = entry.0.unwrap();
                UrlFinalityCheckpointResp {
                    url: entry.1.clone(),
                    data: entry_0.data
                }
            })
    });

    hash_to_value_map
}