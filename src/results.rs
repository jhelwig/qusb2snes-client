use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Result {
  pub results: ResultData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultData {
  Text(Vec<String>),
  Binary(Vec<u8>),
}
