use crate::gateway::*;
use crate::utils::*;
use clap::{Parser, Subcommand};
use std::fs;
use std::{thread, time};
use transaction::prelude::*;

// Enkinet network data
const NETWORK_ID: u8 = 0x21;
const NETWORK_NAME: &str = "enkinet";
const NETWORK_HRP_SUFFIX: &str = "tdx_21_";
const GATEWAY_URL: &str = "https://enkinet-gateway.radixdlt.com";

const CRYPTO_SCRYPTO_BLUEPRINT_NAME: &str = "CryptoScrypto";

// This is the package address of the published CryptoScrypto blueprint.
// If you publish it by yourself you can use the new adress as well.
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
            logical_name: String::from(NETWORK_NAME),
            hrp_suffix: String::from(NETWORK_HRP_SUFFIX),
        };
        let address_decoder = AddressBech32Decoder::new(&network_definition);
        let hash_encoder = TransactionHashBech32Encoder::new(&network_definition);

        // Key must be generated randomly.
        // For the sake of the simplicity we derive it from hardcoded integer.
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

        // Intent hash (unique identifier), which is often used
        // to query it's status in the gateway or in the dashboard.
        // It must be converted to Bech32 format before.
        // Eg.
        //   txid_tdx_21_14a9mm2e3fxyyh02wrz4xsalyxszez6kpqfh0a488hp8wjdvv55cq3wfzv0
        let intent_hash = self.hash_encoder.encode(&intent_hash).unwrap();
        println!("intent_hash: {}", intent_hash);

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

    // Call CryptoScrypto package "keccak256_hash" method to retrieve the digest of the message.
    fn cmd_keccak_hash(&self, cmd: &KeccakHash) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();

        let data = match (&cmd.file_path, &cmd.text) {
            (Some(path), _) => fs::read(path).unwrap(),
            (None, Some(text)) => text.as_bytes().to_vec(),
            (None, None) => panic!("No data given"),
        };

        // Build manifest
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
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        let output = details.get_output(1);

        // The data is in an SBOR encode in hex string.
        // We need to decode it:
        // - first to raw SBOR (byte array)
        // - then decode SBOR to the expected type
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
