use serde::Deserialize;
use toml;


#[derive(Debug, Default, Deserialize)]
pub struct Unit {
    name: String,
    description: String,
    depends : Option<Vec<String>>,
    after: Option<Vec<String>>,
    exec: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    unit : Option<Vec<Unit>>,
}

