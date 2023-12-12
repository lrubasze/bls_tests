use crate::gateway::*;
use crate::utils::*;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::{thread, time};
use transaction::prelude::*;

// Enkinet network ID
const NETWORK_ID: u8 = 0x21;
const GATEWAY_URL: &str = "https://enkinet-gateway.radixdlt.com";

const CRYPTO_SCRYPTO_BLUEPRINT_NAME: &str = "CryptoScrypto";
const CRYPTO_SCRYPTO_PACKAGE_ADDRESS: &str =
    "package_tdx_21_1pkt7zdllsneytdc9g60xn9jjhhwx7jaqxmeh58l4dwyx7rt5z9428f";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GatewayStatus,
    KeccakHash(KeccakHash),
}

#[derive(Debug, Parser)]
struct KeccakHash {
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    package_address: String,
    #[arg(long, short)]
    file_path: Option<String>,
    #[arg(long, short)]
    text: Option<String>,
}

struct CliCtx {
    gateway: GatewayApiClient,
    network_definition: NetworkDefinition,
    address_decoder: AddressBech32Decoder,
    hash_encoder: TransactionHashBech32Encoder,
    private_key: Secp256k1PrivateKey,
}

impl CliCtx {
    fn new() -> Self {
        let gateway = GatewayApiClient::new(GATEWAY_URL);
        let network_definition = NetworkDefinition {
            id: NETWORK_ID,
            logical_name: String::from("enkinet"),
            hrp_suffix: String::from("tdx_21_"),
        };
        let address_decoder = AddressBech32Decoder::new(&network_definition);
        let hash_encoder = TransactionHashBech32Encoder::new(&network_definition);
        let private_key = Secp256k1PrivateKey::from_u64(3).unwrap();
        Self {
            gateway,
            network_definition,
            address_decoder,
            hash_encoder,
            private_key,
        }
    }

    fn cmd_gateway_status(&self) {
        let status = self.gateway.gateway_status();
        println!("gw status = {:?}", status);
    }

    fn execute_transaction(&self, manifest: TransactionManifestV1) -> TransactionDetails {
        let current_epoch = self.gateway.current_epoch();
        let (notarized_transaction, intent_hash) =
            create_notarized_transaction(&self.network_definition, current_epoch, &self.private_key, manifest);

        let _ = self.gateway.transaction_submit(notarized_transaction);

        let intent_hash = self.hash_encoder.encode(&intent_hash).unwrap();

        // Wait for transaction finish
        loop {
            let status = self.gateway.transaction_status(&intent_hash);
            if !status.status.eq("Pending") {
                break;
            }
            thread::sleep(time::Duration::from_millis(1000));
        }
        self.gateway.transaction_details(&intent_hash)
    }

    fn cmd_keccak_hash(&self, cmd: &KeccakHash) {
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();
        let data = match (&cmd.file_path, &cmd.text) {
            (Some(path), _) => fs::read(path).unwrap(),
            (None, Some(text)) => text.as_bytes().to_vec(),
            (None, None) => panic!("No data given"),
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "keccak256_hash",
                manifest_args!(&data),
            )
            .build();

        let details = self.execute_transaction(manifest);
        let output = details.get_output(1);
        let sbor_data = hex::decode(output).unwrap();

        let h: Hash = scrypto_decode(&sbor_data).unwrap();
        println!("hash = {:?}", h);
    }
}

pub fn run() {
    let cli = Cli::parse();

    let ctx = CliCtx::new();

    match &cli.command {
        Commands::GatewayStatus => {
            ctx.cmd_gateway_status();
        }
        Commands::KeccakHash(cmd) => {
            ctx.cmd_keccak_hash(cmd);
        }
    }
}
