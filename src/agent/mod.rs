use rand::RngExt;
use rand::distr::Alphanumeric;

pub mod cmd;

pub fn new_agent_id(role: &Option<String>) -> String {
  let dsc: String = rand::rng()
    .sample_iter(Alphanumeric)
    .take(16)
    .map(char::from)
    .collect();
  match role {
    Some(role) => format!("agent-{}-{}", role, &dsc),
    None       => format!("agent-{}", &dsc),
  }
}
