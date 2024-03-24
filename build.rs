use bwenv_lib::config_yaml;
use schemars::schema_for;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=lib/src/config_yaml.rs");

    let schema = schema_for!(config_yaml::Config);
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    fs::write("schema.json", schema_str).unwrap();
}
