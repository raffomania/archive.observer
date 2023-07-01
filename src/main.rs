use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn main() {
    let filter = EnvFilter::from_default_env();
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    aharc::run().unwrap();
}
