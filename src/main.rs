//mod error;
mod cli;
mod gateway;
mod utils;

// Enkinet network ID
pub const NETWORK_ID: u8 = 0x21;
pub const GATEWAY_URL: &str = "https://enkinet-gateway.radixdlt.com";

fn main() {
    cli::run()
}
