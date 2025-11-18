use eyre::Result;
use clap::Parser;
use peer_practice::input;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[tokio::main]
async fn main() -> Result<()> {
    let cli = input::App::parse();

    cli.run().await
}
