//! JSON Schema Draft-07 official test suite runner.
//!
//! Loads test cases from `../../draft-07/*.json` (sourced from
//! https://github.com/json-schema-org/JSON-Schema-Test-Suite) and runs
//! each through our validation engine.

use schema2object_codegen::spec::ObjectTree;
use serde_json::Value;
use std::fs;
use std::path::Path;

const REMOTE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../draft-07-remotes");

/// A single test group from the test suite.
#[derive(serde::Deserialize)]
struct TestGroup {
    description: String,
    schema: Value,
    tests: Vec<TestCase>,
}

/// A single test case within a group.
#[derive(serde::Deserialize)]
struct TestCase {
    description: String,
    data: Value,
    valid: bool,
}

/// Run all test cases from a single JSON file, returning (pass, fail, skip) counts.
fn run_file(path: &Path) -> (usize, usize, Vec<String>) {
    let content = fs::read_to_string(path).expect("failed to read test file");
    let groups: Vec<TestGroup> =
        serde_json::from_str(&content).expect("failed to parse test file");

    let file_name = path.file_stem().unwrap().to_str().unwrap();
    let mut pass = 0;
    let mut failures = Vec::new();

    for group in &groups {
        for case in &group.tests {
            let result = ObjectTree::try_new(
                case.data.clone(),
                group.schema.clone(),
                Some(REMOTE_DIR.to_string()),
            );
            let actual_valid = result.is_ok();

            if actual_valid == case.valid {
                pass += 1;
            } else {
                failures.push(format!(
                    "  {file_name} / {} / {}: expected valid={}, got valid={} {}",
                    group.description,
                    case.description,
                    case.valid,
                    actual_valid,
                    if let Err(e) = &result {
                        format!("(error: {})", e)
                    } else {
                        String::new()
                    }
                ));
            }
        }
    }

    (pass, failures.len(), failures)
}

#[test]
fn draft07_test_suite() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../draft-07");
    if !dir.exists() {
        eprintln!("Draft-07 test suite not found at {}, skipping", dir.display());
        return;
    }

    let mut total_pass = 0;
    let mut total_fail = 0;
    let mut all_failures = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("failed to read draft7 dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let (pass, fail, failures) = run_file(&path);
        let name = path.file_stem().unwrap().to_str().unwrap();
        if fail > 0 {
            eprintln!("{name}: {pass} pass, {fail} FAIL");
        } else {
            eprintln!("{name}: {pass} pass");
        }
        total_pass += pass;
        total_fail += fail;
        all_failures.extend(failures);
    }

    eprintln!("\n=== Draft-07 Suite Total: {total_pass} pass, {total_fail} fail ===\n");

    if !all_failures.is_empty() {
        eprintln!("Failures:");
        for f in &all_failures {
            eprintln!("{f}");
        }
    }

    // Allow some failures for features we intentionally don't support (e.g., $ref)
    // but assert a high pass rate
    let total = total_pass + total_fail;
    let rate = total_pass as f64 / total as f64 * 100.0;
    eprintln!("\nPass rate: {rate:.1}%");
    assert!(
        rate >= 100.0,
        "Draft-07 pass rate {rate:.1}% is below 100% threshold ({total_fail} failures)"
    );
}
