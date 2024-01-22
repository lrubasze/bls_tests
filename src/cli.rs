use crate::gateway::*;
use crate::utils::*;
use clap::{Parser, Subcommand};
use scrypto::blueprints::package::PackageDefinition;
use std::fs;
use std::{thread, time};
use transaction::prelude::*;

// Enkinet network data
const NETWORK_ID: u8 = 0x21;
const NETWORK_NAME: &str = "enkinet";
const NETWORK_HRP_SUFFIX: &str = "tdx_21_";
const GATEWAY_URL: &str = "https://enkinet-gateway.radixdlt.com";

// Mardunet network data
const MARDUNET_NETWORK_ID: u8 = 0x24;
const MARDUNET_NETWORK_NAME: &str = "mardunet";
const MARDUNET_NETWORK_HRP_SUFFIX: &str = "tdx_24_";
const MARDUNET_GATEWAY_URL: &str = "https://mardunet-gateway.radixdlt.com";

const CRYPTO_SCRYPTO_BLUEPRINT_NAME: &str = "CryptoScrypto";

// This is the package address of the published CryptoScrypto blueprint.
// If you publish it by yourself you can use the new adress as well.
const CRYPTO_SCRYPTO_PACKAGE_ADDRESS: &str =
    "package_tdx_21_1pkt7zdllsneytdc9g60xn9jjhhwx7jaqxmeh58l4dwyx7rt5z9428f";

const TEST_MSG: &str = "Hello World!";
const _TEST_MSG_HASH: &str = "3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0";
// Below key is derived from secret key: 5B00CC8C7153F39EF2E6E2FADB1BB95A1F4BF21F43CC5B28EFA9E526FB788C08
const TEST_PUB_KEY: &str = "8a38419cb83c15a92d11243384bea0acd15cbacc24b385b9c577b17272d6ad68bb53c52dbbf79324005528d2d73c2643";
const TEST_SIGNATURE: &str = "82131f69b6699755f830e29d6ed41cbf759591a2ab598aa4e9686113341118d1db900d190436048601791121b5757c341045d4d0c94a95ec31a9ba6205f9b7504de85dadff52874375c58eec6cec28397279de87d5595101e398d31646d345bb";

const CRYPTO_SCRYPTO_CODE_PATH: &str = "crypto_scrypto/crypto_scrypto.wasm";
const CRYPTO_SCRYPTO_RPD_PATH: &str = "crypto_scrypto/crypto_scrypto.rpd";
const CRYPTO_SCRYPTO_METADATA: &str = "CryptoScrypto package for Supra";

#[derive(Parser)]
#[command(author, version, about, long_about, verbatim_doc_comment)]
#[command(propagate_version = true)]
/// Simple CLI tool to demonstrate how to work with Scrypto blueprints using Rust language.
///
/// It communicates with the network via Gateway using HTTP REST API.
/// It performs:
/// - building transaction manifest for given command
/// - signing the transaction
/// - submitting the transaction to the network
/// - getting the transaction output
struct Cli {
    #[arg(long, short, default_value_t = NETWORK_NAME.to_string())]
    /// Switch to mardunet network
    network: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get gateway status. This is sanity check, whether gateway is working fine.
    GatewayStatus,
    /// Calculate Keccak256 hash over given message
    KeccakHash(KeccakHash),
    /// Perform BLS verification over Keccak256 hash from given message, public key and signature
    BlsVerify(BlsVerify),
    /// Perform BLS aggregate verification over given messages, public keys and signature
    BlsAggregateVerify(BlsAggregateVerify),
    /// Perform BLS aggregate verification over given message, public keys and signature
    BlsFastAggregateVerify(BlsFastAggregateVerify),
    /// Perform BLS signature aggregation
    BlsSignatureAggregate(BlsSignatureAggregate),
    /// Publish given WASM and RPD files as a package
    PublishPackage(PublishPackage),
}

#[derive(Debug, Parser)]
struct KeccakHash {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, default_value_t = TEST_MSG.to_string())]
    /// Message to hash
    msg: String,
}

#[derive(Debug, Parser)]
struct BlsVerify {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, default_value_t = TEST_MSG.to_string())]
    /// Message to verify signature with (it will be hashed before with Keccak256)
    msg: String,
    #[arg(long, short, default_value_t = TEST_PUB_KEY.to_string())]
    /// BLS public key to perform verification (hex-encoded string)
    public_key: String,
    /// BLS signature to verify (hex-encoded string)
    #[arg(long, short, default_value_t = TEST_SIGNATURE.to_string())]
    signature: String,
}

#[derive(Debug, Parser)]
struct BlsAggregateVerify {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, use_value_delimiter = true, value_delimiter = ',', default_values_t = vec![TEST_MSG.to_string()])]
    /// Messages to verify signature with (it will be hashed before with Keccak256)
    msgs: Vec<String>,
    #[arg(long, short, use_value_delimiter = true, value_delimiter = ',', default_values_t = vec![TEST_PUB_KEY.to_string()])]
    /// BLS public key to perform verification (hex-encoded string)
    public_keys: Vec<String>,
    /// BLS signature to verify (hex-encoded string)
    #[arg(long, short, default_value_t = TEST_SIGNATURE.to_string())]
    signature: String,
}

#[derive(Debug, Parser)]
struct BlsFastAggregateVerify {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, default_value_t = TEST_MSG.to_string())]
    /// Message to verify signature with (it will be hashed before with Keccak256)
    msg: String,
    #[arg(long, short, use_value_delimiter = true, value_delimiter = ',', default_values_t = vec![TEST_PUB_KEY.to_string()])]
    /// BLS public key to perform verification (hex-encoded string)
    public_keys: Vec<String>,
    /// BLS signature to verify (hex-encoded string)
    #[arg(long, short, default_value_t = TEST_SIGNATURE.to_string())]
    signature: String,
}

#[derive(Debug, Parser)]
struct BlsSignatureAggregate {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    /// BLS signatures to aggregate (hex-encoded string)
    #[arg(long, short, use_value_delimiter = true, value_delimiter = ',', default_values_t = vec![TEST_SIGNATURE.to_string()])]
    signatures: Vec<String>,
}

#[derive(Debug, Parser)]
struct PublishPackage {
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_CODE_PATH.to_string())]
    /// Scrypto blueprint WASM file, output of 'scrypto build' command
    code_path: String,
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_RPD_PATH.to_string())]
    /// Scrypto blueprint package definition file, output of 'scrypto build' command
    rpd_path: String,
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_METADATA.to_string())]
    /// Package metadata to set for 'Description' key
    metadata: String,
}

struct CliCtx {
    gateway: GatewayApiClient,
    network_definition: NetworkDefinition,
    address_decoder: AddressBech32Decoder,
    address_encoder: AddressBech32Encoder,
    hash_encoder: TransactionHashBech32Encoder,
    private_key: Secp256k1PrivateKey,
}

impl CliCtx {
    fn new(network_name: &str) -> Self {
        let (gateway, network_definition) = match network_name {
            MARDUNET_NETWORK_NAME => (
                GatewayApiClient::new(MARDUNET_GATEWAY_URL),
                NetworkDefinition {
                    id: MARDUNET_NETWORK_ID,
                    logical_name: String::from(MARDUNET_NETWORK_NAME),
                    hrp_suffix: String::from(MARDUNET_NETWORK_HRP_SUFFIX),
                },
            ),
            NETWORK_NAME => (
                GatewayApiClient::new(GATEWAY_URL),
                NetworkDefinition {
                    id: NETWORK_ID,
                    logical_name: String::from(NETWORK_NAME),
                    hrp_suffix: String::from(NETWORK_HRP_SUFFIX),
                },
            ),
            _ => panic!("Network '{}' not supported", network_name),
        };
        let address_decoder = AddressBech32Decoder::new(&network_definition);
        let address_encoder = AddressBech32Encoder::new(&network_definition);
        let hash_encoder = TransactionHashBech32Encoder::new(&network_definition);

        // Key must be generated randomly.
        // For the sake of the simplicity we derive it from hardcoded integer.
        let private_key = Secp256k1PrivateKey::from_u64(3).unwrap();
        Self {
            gateway,
            network_definition,
            address_decoder,
            address_encoder,
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

        let (notarized_transaction, intent_hash) = create_notarized_transaction(
            &self.network_definition,
            current_epoch,
            &self.private_key,
            manifest,
        );

        // Intent hash (unique identifier), which is often used
        // to query it's status in the gateway or in the dashboard.
        // It must be converted to Bech32 format before.
        // Eg.
        //   txid_tdx_21_14a9mm2e3fxyyh02wrz4xsalyxszez6kpqfh0a488hp8wjdvv55cq3wfzv0
        let intent_hash = self.hash_encoder.encode(&intent_hash).unwrap();
        println!("intent_hash : {}", intent_hash);

        let submit = self.gateway.transaction_submit(notarized_transaction);
        if let Some(message) = submit.message {
            println!("Transaction submit error");
            println!("message: {}", message);
            println!("code: {:?}", submit.code.unwrap());
            println!("details: {:?}", submit.details.unwrap());
            panic!("")
        }

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
        let data = cmd.msg.as_bytes().to_vec();

        println!("Package address : {}", cmd.package_address);
        println!("Message         : {}", cmd.msg);

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
        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let hash: Hash = scrypto_decode(&sbor_data).unwrap();
            println!("Message hash    : {}", hash);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }

    // Call CryptoScrypto package "bls12381_v1_verify" method to verify the signature
    fn cmd_bls_verify(&self, cmd: &BlsVerify) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();
        let msg_hash = keccak256_hash(cmd.msg.clone());

        println!("Package address : {}", cmd.package_address);
        println!("Message         : {}", cmd.msg);
        println!("Message hash    : {}", msg_hash);
        println!("Publick key     : {}", cmd.public_key);
        println!("Signature       : {}", cmd.signature);

        let pub_key = Bls12381G1PublicKey::from_str(&cmd.public_key).unwrap();
        let signature = Bls12381G2Signature::from_str(&cmd.signature).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "bls12381_v1_verify",
                manifest_args!(msg_hash.to_vec(), pub_key, signature),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let result: bool = scrypto_decode(&sbor_data).unwrap();
            println!("BLS verify  : {:?}", result);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }

    // Publish package using given *.wasm and *.rpd files
    fn cmd_publish_package(&self, cmd: &PublishPackage) {
        println!("WASM file: {}", cmd.code_path);
        println!("RPD file : {}", cmd.rpd_path);
        println!("Metadata : {}", cmd.metadata);

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "Description".to_string(),
            MetadataValue::String(cmd.metadata.to_string()),
        );
        let code = fs::read(cmd.code_path.clone()).unwrap();
        let rpd: PackageDefinition =
            manifest_decode(&fs::read(cmd.rpd_path.clone()).unwrap()).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, rpd, metadata, OwnerRole::None)
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let address: PackageAddress = scrypto_decode(&sbor_data).unwrap();

            // Encode the address into human-readabl bech32 format
            let address = self.address_encoder.encode(address.as_ref()).unwrap();
            println!("Published package address  : {}", address);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }

    fn cmd_bls_aggregate_verify(&self, cmd: &BlsAggregateVerify) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();

        println!("Package address : {}", cmd.package_address);
        println!("Messages        : {:?}", cmd.msgs);
        println!("Public  keys    : {:?}", cmd.public_keys);
        println!("Signature       : {:?}", cmd.signature);

        let pub_keys_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)> = cmd
            .public_keys
            .iter()
            .zip(cmd.msgs.clone())
            .map(|(pk, msg)| (Bls12381G1PublicKey::from_str(pk).unwrap(), msg.into_bytes()))
            .collect();

        let signature = Bls12381G2Signature::from_str(&cmd.signature).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "bls12381_v1_aggregate_verify",
                manifest_args!(pub_keys_msgs, signature),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let result: bool = scrypto_decode(&sbor_data).unwrap();
            println!("BLS aggregate verify  : {:?}", result);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }

    fn cmd_bls_fast_aggregate_verify(&self, cmd: &BlsFastAggregateVerify) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();

        println!("Package address : {}", cmd.package_address);
        println!("Message         : {:?}", cmd.msg);
        println!("Public keys     : {:?}", cmd.public_keys);
        println!("Signature       : {:?}", cmd.signature);

        let msg = cmd.msg.clone().into_bytes();
        let pub_keys: Vec<Bls12381G1PublicKey> = cmd
            .public_keys
            .iter()
            .map(|pk| Bls12381G1PublicKey::from_str(pk).unwrap())
            .collect();

        let signature = Bls12381G2Signature::from_str(&cmd.signature).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "bls12381_v1_fast_aggregate_verify",
                manifest_args!(msg, pub_keys, signature),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails

        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let result: bool = scrypto_decode(&sbor_data).unwrap();
            println!("BLS fast aggregate verify  : {:?}", result);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }

    fn cmd_bls_signature_aggregate(&self, cmd: &BlsSignatureAggregate) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();

        println!("Package address : {}", cmd.package_address);
        println!("Signatures      : {:?}", cmd.signatures);

        let signatures: Vec<Bls12381G2Signature> = cmd
            .signatures
            .iter()
            .map(|s| Bls12381G2Signature::from_str(s).unwrap())
            .collect();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "bls12381_g2_signature_aggregate",
                manifest_args!(signatures),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        if let Some(output) = details.get_output(1) {
            // The data is in an SBOR encode in hex string.
            // We need to decode it:
            // - first to raw SBOR (byte array)
            // - then decode SBOR to the expected type
            let sbor_data = hex::decode(output).unwrap();

            let result: bool = scrypto_decode(&sbor_data).unwrap();
            println!("BLS signature aggregate  : {:?}", result);
        } else {
            let error = details.get_error().unwrap();
            println!("Transaction error: {:?}", error);
        }
    }
}

pub fn run() {
    let cli = Cli::parse();

    let ctx = CliCtx::new(&cli.network);

    match &cli.command {
        Commands::GatewayStatus => {
            ctx.cmd_gateway_status();
        }
        Commands::KeccakHash(cmd) => {
            ctx.cmd_keccak_hash(cmd);
        }
        Commands::BlsVerify(cmd) => {
            ctx.cmd_bls_verify(cmd);
        }
        Commands::BlsAggregateVerify(cmd) => {
            ctx.cmd_bls_aggregate_verify(cmd);
        }
        Commands::BlsFastAggregateVerify(cmd) => {
            ctx.cmd_bls_fast_aggregate_verify(cmd);
        }
        Commands::BlsSignatureAggregate(cmd) => {
            ctx.cmd_bls_signature_aggregate(cmd);
        }
        Commands::PublishPackage(cmd) => {
            ctx.cmd_publish_package(cmd);
        }
    }
}
