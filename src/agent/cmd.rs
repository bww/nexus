use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::agent;
use crate::Options;

#[derive(Args, Debug)]
pub struct AgentOptions {
  #[clap(subcommand)]
  command: AgentCommand,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
  #[clap(name="new", about="Create a new agent identifier")]
  Create(CreateAgentOptions),
}

#[derive(Args, Debug)]
pub struct CreateAgentOptions {
  #[clap(long, help="The agent's role")]
  role: String,
}

pub fn agent(opts: &Options, agent: &AgentOptions, conn: Connection) -> Result<(), error::Error> {
  match &agent.command {
    AgentCommand::Create(sub) => create_agent(opts, agent, sub, conn),
  }
}

fn create_agent(_opts: &Options, _agent: &AgentOptions, create: &CreateAgentOptions, conn: Connection) -> Result<(), error::Error> {
  println!(">>> {}", agent::new_agent_id(&create.role));
  Ok(())
}
