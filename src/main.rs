use clap::Parser;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn main() {
    let filter = EnvFilter::from_default_env();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let config = aharc::config::Config::parse();

    aharc::run(config).unwrap();
}
