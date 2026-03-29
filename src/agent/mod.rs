use rand::RngExt;
use rand::distr::Alphanumeric;

pub mod cmd;

pub fn new_agent_id(role: &str) -> String {
  let dsc: String = rand::rng()
    .sample_iter(Alphanumeric)
    .take(16)
    .map(char::from)
    .collect();
  format!("agent-{}-{}", role, &dsc)
}
