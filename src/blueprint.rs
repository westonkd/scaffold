use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BlueprintConfig {
    pub name: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub source: String,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
    pub path: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
}
