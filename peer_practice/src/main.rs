use clap::Parser;
use eyre::Result;
use mimalloc::MiMalloc;
use peer_practice::input;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[tokio::main]
async fn main() -> Result<()> {
    let cli = input::App::parse();

    cli.run().await
}
