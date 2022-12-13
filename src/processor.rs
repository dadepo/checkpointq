use std::collections::{BTreeMap, HashMap};
use futures::StreamExt;
use crate::client::{Finality_Results, UrlFinalityCheckpointResp};


pub fn process_result(raw_result: Finality_Results) -> () {
    let (successes, failures): (Finality_Results, Finality_Results) = raw_result.into_iter().partition(|raw| {
        raw.0.is_ok()
    });

    let mut success_map: HashMap<String, Vec<UrlFinalityCheckpointResp>> = HashMap::new();
    successes.into_iter().for_each(|success| {
        success_map.entry(success.0.as_ref().unwrap().data.finalized.root.clone())
            .and_modify(|e| e.push(UrlFinalityCheckpointResp {
                url: success.1.clone(),
                data: success.0.as_ref().unwrap().data.clone()
            })).or_insert_with(|| {
            vec![UrlFinalityCheckpointResp {
                url: success.1.clone(),
                data: success.0.unwrap().data
            }]
        });
    });

    println!("{:?}", success_map);
    println!("{:?}", success_map.keys());
    // dbg!(success);
    // dbg!(failures);
}