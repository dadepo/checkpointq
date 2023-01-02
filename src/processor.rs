use crate::client::{ResponsePayloadWithEndpointInfo, SuccessEndpointPayload};
use crate::errors::AppError;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct GroupedResult {
    success: HashMap<String, Vec<SuccessPayload>>,
    failure: Vec<FailurePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayableResult {
    pub canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub non_canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub failure: Vec<FailurePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessPayload {
    pub payload: SuccessEndpointPayload,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailurePayload {
    pub payload: AppError,
    pub endpoint: String,
}

fn group_success_failure(response_payload: Vec<ResponsePayloadWithEndpointInfo>) -> GroupedResult {
    let (successes, failures): (Vec<SuccessPayload>, Vec<FailurePayload>) = response_payload
        .into_iter()
        .fold((vec![], vec![]), |mut acc, result| {
            match result.payload {
                Ok(success) => acc.0.push(SuccessPayload {
                    payload: success,
                    endpoint: result.endpoint,
                }),
                Err(error) => acc.1.push(FailurePayload {
                    payload: error,
                    endpoint: result.endpoint,
                }),
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
        failure: failures,
    }
}

pub fn process_to_displayable_format(
    response_payload: Vec<ResponsePayloadWithEndpointInfo>,
) -> DisplayableResult {
    // groups the results into
    // failures
    // success
    //   - success grouped by their block_root hash
    let grouped_result = group_success_failure(response_payload);
    let mut canonical: Option<HashMap<String, Vec<SuccessPayload>>> = None;
    let mut non_canonical: Option<HashMap<String, Vec<SuccessPayload>>> = None;

    if !grouped_result.success.is_empty() {
        if grouped_result.success.keys().len() == 1 {
            canonical = Some(grouped_result.success);
        } else {
            // more than one results, pick one with values more than 2/3
            let total_value = grouped_result.success.values().len() as f64;
            let threshold = (2f64 / 3f64 * total_value).floor();
            let (passed_threshold, below_threshold): (
                HashMap<String, Vec<SuccessPayload>>,
                HashMap<String, Vec<SuccessPayload>>,
            ) = grouped_result
                .success
                .into_iter()
                .partition(|(_, values)| values.len() as f64 > threshold);
            if passed_threshold.keys().len() == 1 {
                // if there is only one value thay passed the threshold that is the canonical result
                canonical = Some(passed_threshold)
            } else {
                // else the non_canonical will include
                // the multiple values that passed the threshold
                // the values that did not even pass the threshold
                non_canonical = Some(
                    passed_threshold
                        .into_iter()
                        .chain(below_threshold)
                        .collect(),
                )
            }
        }
    };

    DisplayableResult {
        canonical,
        non_canonical,
        failure: grouped_result.failure,
    }
}

pub fn print_result(result: DisplayableResult, is_verbose: bool) {
    if let Some(canonical_result) = result.canonical {
        println!(
            "{}:\n \t{}",
            "Block root".blue(),
            canonical_result
                .keys()
                .next()
                .unwrap_or(&"block root not found".to_string())
                .green()
                .bold()
        );

        if is_verbose {
            println!(
                "{}:\n \t{}",
                "Details".blue(),
                serde_json::to_string_pretty(&canonical_result)
                    .unwrap_or("displaying verbose failed".to_string())
                    .green()
            );
        }
    };

    if let Some(non_canonical_result) = result.non_canonical {
        println!("{}", "Conflicting:".yellow().bold());
        if is_verbose {
            println!(
                "{}:\n \t{}",
                "Details".yellow(),
                serde_json::to_string_pretty(&non_canonical_result)
                    .unwrap_or("displaying verbose failed".to_string())
                    .yellow()
            );
        } else {
            for (key, values) in &non_canonical_result {
                println!("\t Checkpoint: {}", key.yellow());
                for value in values {
                    println!("\t\t {}", value.endpoint.yellow());
                }
            }
        }
    }

    if !result.failure.is_empty() {
        println!("{}", "Errors:".red().bold());
        if is_verbose {
            println!(
                "{}:\n \t{}",
                "Details".red(),
                serde_json::to_string_pretty(&result.failure)
                    .unwrap_or("displaying error failed".to_string())
                    .red()
            );
        } else {
            result.failure.into_iter().for_each(|failure_value| {
                println!("\t Endpoint: {}", failure_value.endpoint.red());
                println!("\t Error: {}", failure_value.payload.to_string().red());
            });
        }
    }
}
