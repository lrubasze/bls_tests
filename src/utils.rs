use transaction::prelude::*;

pub fn create_notarized_transaction(
    network_definition: &NetworkDefinition,
    epoch: u64,
    manifest: TransactionManifestV1,
) -> (NotarizedTransactionV1, IntentHash) {
    let sk_notary = Secp256k1PrivateKey::from_u64(3).unwrap();
    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network_definition.id,
            start_epoch_inclusive: Epoch::of(epoch),
            end_epoch_exclusive: Epoch::of(epoch + 10),
            nonce: 5,
            notary_public_key: sk_notary.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .notarize(&sk_notary)
        .build();

    let intent_hash = transaction.prepare().unwrap().intent_hash();

    (transaction, intent_hash)
}
