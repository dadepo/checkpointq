use std::collections::{HashMap};
use crate::args::DisplayLevel;
use crate::client::{DisplayableResult, FailurePayload, GroupedResult, ResponsePayload, SuccessPayload};
use colored::*;

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

pub fn to_displayable_result(grouped_result: GroupedResult) -> DisplayableResult {
    let mut canonical: Option<HashMap<String, Vec<SuccessPayload>>> = None;
    let mut non_canonical: Option<HashMap<String, Vec<SuccessPayload>>> = None;
    let mut failure: Vec<FailurePayload> = vec![];

    if !grouped_result.success.is_empty() {
      if grouped_result.success.keys().len() == 1 {
          canonical = Some(grouped_result.success);
          failure = grouped_result.failure;
      } else {
          // more than one results, pick one with values more than 2/3
          let total_value = (grouped_result.success.values().len() as f64);
          let threshold = 2f64/3f64 * total_value;
          let possible_canonical_results: HashMap<String, Vec<SuccessPayload>> =
              grouped_result
                  .success
                  .into_iter()
                  .filter(|(key, values)| {
                      values.len() as f64 > threshold
                  }).collect();
          if possible_canonical_results.keys().len() == 1 {
              canonical = Some(possible_canonical_results)
          } else {
              non_canonical = Some(possible_canonical_results)
          }
      }
    } else {
        failure = grouped_result.failure;
    };
    DisplayableResult {
        canonical,
        non_canonical,
        failure
    }
}

pub fn display_result(displayable_result: DisplayableResult, display_level: DisplayLevel) {
    match display_level {
        DisplayLevel::Normal => normal_result(displayable_result),
        DisplayLevel::Verbose => normal_result(displayable_result),
    }
}

fn normal_result(displayable_result: DisplayableResult) {
    if let Some(canonical_result) = displayable_result.canonical {
        println!("{}:\n \t{}", "Checkpoint".blue(), canonical_result.keys().next().unwrap().green().bold())
    };

    if let Some(non_canonical_result) = displayable_result.non_canonical {
        println!("{}", "Conflicting:".yellow().bold());
        for (key, values) in non_canonical_result {
            println!("\t Checkpoint: {}", key.yellow());
            for value in values {
                println!("\t\t {}", value.endpoint.yellow());
            }
        }
    }

    if !displayable_result.failure.is_empty() {
        println!("{}", "Errors:".red().bold());
    }
    displayable_result.failure.into_iter().for_each(|failure_value| {
        println!("\t Endpoint: {}", failure_value.endpoint.red());
        println!("\t Error: {}", failure_value.payload.to_string().red());
    });
}