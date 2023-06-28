/*!
Provide a crate wide configuration singleton.
Data is sourced from environment variables.
!*/

use once_cell::sync::Lazy;
use serde::Deserialize;

/// Configuration variables for the crate.
#[derive(Deserialize, Debug)]
pub struct Config {}

/// Access to parsed configuration.
pub static CONFIG: Lazy<Config> = Lazy::new(|| envy::from_env().expect("some env vars missing"));
