/*!
Provide a crate wide configuration singleton.
Data is sourced from environment variables.
!*/

use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

/// Configuration variables for the crate.
#[derive(Deserialize, Debug, Parser)]
#[command()]
pub struct Config {
    /// Path to the json file containing reddit posts.
    #[arg(env, long)]
    pub submissions: PathBuf,
    /// Path to the json file containing reddit comments.
    #[arg(env, long)]
    pub comments: PathBuf,
    /// Limit the number of rendered posts to this value, if set.
    /// Use this to speed up the rendering in development or testing.
    #[arg(env, long)]
    pub limit_posts: Option<usize>,
}
