use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use dotenv_parser::parse_dotenv;

#[test]
fn missing_token() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    // cmd.arg("foobar").arg("test/file/doesnt/exist");

    cmd.assert().failure().stderr(predicate::str::contains(
        "The following required argument was not provided: token",
    ));

    Ok(())
}

fn parse_env() -> std::collections::BTreeMap<String, String> {
    parse_dotenv(std::fs::read_to_string(".env").unwrap().as_str()).unwrap_or_else(|_| {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            String::from("BWS_ACCESS_TOKEN"),
            std::env::var("BWS_ACCESS_TOKEN").expect("failed to read access token"),
        );
        map
    })
}

#[test]
fn missing_slop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    cmd.arg("--token")
        .arg(parse_env().get("BWS_ACCESS_TOKEN").unwrap());

    cmd.assert()
        .failure()
        // TODO: this should be stderr
        .stdout(predicate::str::contains("no slop provided"));

    Ok(())
}

#[test]
fn valid_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    cmd.arg("--token")
        .arg(parse_env().get("BWS_ACCESS_TOKEN").unwrap());
    cmd.arg("--").arg("echo test");

    cmd.assert().success();

    Ok(())
}

#[test]
fn prints_values_from_profile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    cmd.arg("--token")
        .arg(parse_env().get("BWS_ACCESS_TOKEN").unwrap())
        .arg("inspect")
        .arg("--reveal");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TEST_VALUE :: default"));

    Ok(())
}

#[test]
fn prints_values_from_global_overrides() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    cmd.arg("--token")
        .arg(parse_env().get("BWS_ACCESS_TOKEN").unwrap())
        .arg("inspect")
        .arg("--reveal");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FORCE_COLOR :: 1"));

    Ok(())
}

#[test]
fn prints_values_custom_profile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bwenv")?;
    cmd.arg("--token")
        .arg(parse_env().get("BWS_ACCESS_TOKEN").unwrap())
        .arg("--profile")
        .arg("other")
        .arg("inspect")
        .arg("--reveal");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TEST_VALUE :: other"));

    Ok(())
}
