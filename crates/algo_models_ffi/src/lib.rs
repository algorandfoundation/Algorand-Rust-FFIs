use algo_models::AlgorandMsgpack;
use ffi_macros::{ffi_func, ffi_record};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

// thiserror is used to easily create errors than can be propagated to the language bindings
// UniFFI will create classes for errors (i.e. `MsgPackError.EncodingError` in Python)
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "ffi_uniffi", derive(uniffi::Error))]
pub enum MsgPackError {
    #[error("EncodingError: {0}")]
    EncodingError(String),
    #[error("DecodingError: {0}")]
    DecodingError(String),
}

// For now, in WASM we just throw the string, hence the error
// type being included in the error string above
// Perhaps in the future we could use a class like in UniFFI
#[cfg(feature = "ffi_wasm")]
impl From<MsgPackError> for JsValue {
    fn from(e: MsgPackError) -> Self {
        JsValue::from(e.to_string())
    }
}

// Convert errors from the Rust crate into the FFI-specific errors
impl From<algo_models::MsgPackError> for MsgPackError {
    fn from(e: algo_models::MsgPackError) -> Self {
        match e {
            algo_models::MsgPackError::DecodeError(_) => MsgPackError::DecodingError(e.to_string()),
            algo_models::MsgPackError::EncodeError(_) => MsgPackError::EncodingError(e.to_string()),
            algo_models::MsgPackError::RmpvDecodeError(_) => {
                MsgPackError::DecodingError(e.to_string())
            }
            algo_models::MsgPackError::RmpvEncodeError(_) => {
                MsgPackError::EncodingError(e.to_string())
            }
            algo_models::MsgPackError::UnknownTransactionType => {
                MsgPackError::DecodingError(e.to_string())
            }
            algo_models::MsgPackError::InputError(e) => MsgPackError::DecodingError(e.to_string()),
        }
    }
}

#[cfg(feature = "ffi_uniffi")]
use uniffi::{self};

#[cfg(feature = "ffi_uniffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "ffi_wasm")]
use tsify_next::Tsify;
#[cfg(feature = "ffi_wasm")]
use wasm_bindgen::prelude::*;

// We need to use ByteBuf directly in the structs to get Uint8Array in TSify
// custom_type! and this impl is used to convert the ByteBuf to a Vec<u8> for the UniFFI bindings
#[cfg(feature = "ffi_uniffi")]
impl UniffiCustomTypeConverter for ByteBuf {
    type Builtin = Vec<u8>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(ByteBuf::from(val))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_vec()
    }
}

#[cfg(feature = "ffi_uniffi")]
uniffi::custom_type!(ByteBuf, Vec<u8>);

// This becomes an enum in UniFFI lnaguage bindings and a
// string literal union in TS
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "ffi_wasm", derive(Tsify))]
#[cfg_attr(feature = "ffi_wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "ffi_uniffi", derive(uniffi::Enum))]
pub enum TransactionType {
    Payment,
    AssetTransfer,
    AssetFreeze,
    AssetConfig,
    KeyRegistration,
    ApplicationCall,
}

// Even though these stucts are all defined in the crate, we need to re-define them
// because we have to use different serde attributes for the struct fields.
// In the crate, we need to use the msgpack names for the fields, but in the FFI
// we need to use the camelCase names for the fields for TSify.

/// The transaction header contains the fields that can be present in any transaction.
/// "Header" only indicates that these are common fields, NOT that they are the first fields in the transaction.
#[ffi_record]
pub struct TransactionHeader {
    /// The type of transaction
    transaction_type: TransactionType,

    /// The sender of the transaction
    sender: ByteBuf,

    fee: u64,

    first_valid: u64,

    last_valid: u64,

    genesis_hash: Option<ByteBuf>,

    genesis_id: Option<String>,

    note: Option<ByteBuf>,

    rekey_to: Option<ByteBuf>,

    lease: Option<ByteBuf>,

    group: Option<ByteBuf>,
}

#[ffi_record]
pub struct PayTransactionFields {
    receiver: ByteBuf,

    amount: u64,

    close_remainder_to: Option<ByteBuf>,
}

#[ffi_record]
pub struct AssetTransferTransactionFields {
    asset_id: u64,

    amount: u64,

    receiver: ByteBuf,

    asset_sender: Option<ByteBuf>,

    close_remainder_to: Option<ByteBuf>,
}

#[ffi_record]
pub struct Transaction {
    header: TransactionHeader,

    pay_fields: Option<PayTransactionFields>,

    asset_transfer_fields: Option<AssetTransferTransactionFields>,
}

impl TryFrom<Transaction> for algo_models::Transaction {
    type Error = MsgPackError;

    fn try_from(tx: Transaction) -> Result<Self, MsgPackError> {
        // Ensure we only have pay fields or asset transfer fields
        let fields: [bool; 2] = [tx.pay_fields.is_some(), tx.asset_transfer_fields.is_some()];

        // If fields has more than one true value, then we have an error
        if fields.iter().filter(|&&x| x).count() > 1 {
            return Err(MsgPackError::DecodingError(
                "Multiple fields set".to_string(),
            ));
        }

        if let Some(pay) = tx.pay_fields {
            return Ok(algo_models::Transaction::Payment(
                algo_models::PayTransactionFields {
                    header: tx.header.try_into()?,
                    amount: pay.amount,
                    receiver: pay.receiver.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "receiver should be 32 byte public key".to_string(),
                        )
                    })?,
                    close_remainder_to: pay
                        .close_remainder_to
                        .map(|b| {
                            b.to_vec().try_into().map_err(|_| {
                                MsgPackError::EncodingError(
                                    "close remainder to should be 32 byte public key".to_string(),
                                )
                            })
                        })
                        .transpose()?,
                },
            ));
        }

        if let Some(asset_transfer) = tx.asset_transfer_fields {
            return Ok(algo_models::Transaction::AssetTransfer(
                algo_models::AssetTransferTransactionFields {
                    header: tx.header.try_into()?,
                    asset_id: asset_transfer.asset_id,
                    amount: asset_transfer.amount,
                    receiver: asset_transfer.receiver.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "receiver should be 32 byte public key".to_string(),
                        )
                    })?,
                    asset_sender: asset_transfer
                        .asset_sender
                        .map(|b| {
                            b.to_vec().try_into().map_err(|_| {
                                MsgPackError::EncodingError(
                                    "close remainder to should be 32 byte public key".to_string(),
                                )
                            })
                        })
                        .transpose()?,
                    close_remainder_to: asset_transfer
                        .close_remainder_to
                        .map(|b| {
                            b.to_vec().try_into().map_err(|_| {
                                MsgPackError::EncodingError(
                                    "close remainder to should be 32 byte public key".to_string(),
                                )
                            })
                        })
                        .transpose()?,
                },
            ));
        }

        Err(MsgPackError::DecodingError(
            "No transaction fields set".to_string(),
        ))
    }
}

impl From<TransactionType> for algo_models::TransactionType {
    fn from(tx: TransactionType) -> Self {
        match tx {
            TransactionType::Payment => algo_models::TransactionType::Payment,
            TransactionType::AssetTransfer => algo_models::TransactionType::AssetTransfer,
            TransactionType::AssetFreeze => algo_models::TransactionType::AssetFreeze,
            TransactionType::AssetConfig => algo_models::TransactionType::AssetConfig,
            TransactionType::KeyRegistration => algo_models::TransactionType::KeyRegistration,
            TransactionType::ApplicationCall => algo_models::TransactionType::ApplicationCall,
        }
    }
}

impl TryFrom<TransactionHeader> for algo_models::TransactionHeader {
    type Error = MsgPackError;

    fn try_from(tx: TransactionHeader) -> Result<Self, MsgPackError> {
        Ok(Self {
            transaction_type: tx.transaction_type.into(),
            sender: tx.sender.to_vec().try_into().map_err(|_| {
                MsgPackError::EncodingError("sender should be 32 byte public key".to_string())
            })?,
            fee: tx.fee,
            first_valid: tx.first_valid,
            last_valid: tx.last_valid,
            genesis_id: tx.genesis_id,
            genesis_hash: tx
                .genesis_hash
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "genesis_hash should be 32 byte hash".to_string(),
                        )
                    })
                })
                .transpose()?,
            note: tx.note.map(|b| b.to_vec()),
            rekey_to: tx
                .rekey_to
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "rekey_to should be 32 byte public key".to_string(),
                        )
                    })
                })
                .transpose()?,
            lease: tx
                .lease
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError("lease should be 32 bytes".to_string())
                    })
                })
                .transpose()?,
            group: tx
                .group
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError("group should be 32 byte hash".to_string())
                    })
                })
                .transpose()?,
        })
    }
}

impl From<algo_models::TransactionHeader> for TransactionHeader {
    fn from(tx: algo_models::TransactionHeader) -> Self {
        Self {
            transaction_type: tx.transaction_type.into(),
            sender: ByteBuf::from(tx.sender.to_vec()),
            fee: tx.fee,
            first_valid: tx.first_valid,
            last_valid: tx.last_valid,
            genesis_id: tx.genesis_id,
            genesis_hash: tx.genesis_hash.map(|b| ByteBuf::from(b.to_vec())),
            note: tx.note.map(|b| ByteBuf::from(b.to_vec())),
            rekey_to: tx.rekey_to.map(|b| ByteBuf::from(b.to_vec())),
            lease: tx.lease.map(|b| ByteBuf::from(b.to_vec())),
            group: tx.group.map(|b| ByteBuf::from(b.to_vec())),
        }
    }
}

impl From<algo_models::PayTransactionFields> for PayTransactionFields {
    fn from(tx: algo_models::PayTransactionFields) -> Self {
        Self {
            receiver: ByteBuf::from(tx.receiver.to_vec()),
            amount: tx.amount,
            close_remainder_to: tx.close_remainder_to.map(|b| ByteBuf::from(b.to_vec())),
        }
    }
}

impl TryFrom<PayTransactionFields> for algo_models::PayTransactionFields {
    type Error = MsgPackError;

    fn try_from(tx: PayTransactionFields) -> Result<Self, Self::Error> {
        Ok(Self {
            header: algo_models::TransactionHeader {
                transaction_type: algo_models::TransactionType::Payment,
                sender: vec![0; 32].try_into().unwrap(), // This will be overridden by the Transaction conversion
                fee: 0,
                first_valid: 0,
                last_valid: 0,
                genesis_id: None,
                genesis_hash: None,
                note: None,
                rekey_to: None,
                lease: None,
                group: None,
            },
            amount: tx.amount,
            receiver: tx.receiver.to_vec().try_into().map_err(|_| {
                MsgPackError::EncodingError("receiver should be 32 byte public key".to_string())
            })?,
            close_remainder_to: tx
                .close_remainder_to
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "close remainder to should be 32 byte public key".to_string(),
                        )
                    })
                })
                .transpose()?,
        })
    }
}

impl From<algo_models::AssetTransferTransactionFields> for AssetTransferTransactionFields {
    fn from(tx: algo_models::AssetTransferTransactionFields) -> Self {
        Self {
            asset_id: tx.asset_id,
            amount: tx.amount,
            receiver: ByteBuf::from(tx.receiver.to_vec()),
            asset_sender: tx.asset_sender.map(|b| ByteBuf::from(b.to_vec())),
            close_remainder_to: tx.close_remainder_to.map(|b| ByteBuf::from(b.to_vec())),
        }
    }
}

impl TryFrom<AssetTransferTransactionFields> for algo_models::AssetTransferTransactionFields {
    type Error = MsgPackError;

    fn try_from(tx: AssetTransferTransactionFields) -> Result<Self, Self::Error> {
        Ok(Self {
            header: algo_models::TransactionHeader {
                transaction_type: algo_models::TransactionType::AssetTransfer,
                sender: vec![0; 32].try_into().unwrap(), // This will be overridden by the Transaction conversion
                fee: 0,
                first_valid: 0,
                last_valid: 0,
                genesis_id: None,
                genesis_hash: None,
                note: None,
                rekey_to: None,
                lease: None,
                group: None,
            },
            asset_id: tx.asset_id,
            amount: tx.amount,
            receiver: tx.receiver.to_vec().try_into().map_err(|_| {
                MsgPackError::EncodingError("receiver should be 32 byte public key".to_string())
            })?,
            asset_sender: tx
                .asset_sender
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "asset sender should be 32 byte public key".to_string(),
                        )
                    })
                })
                .transpose()?,
            close_remainder_to: tx
                .close_remainder_to
                .map(|b| {
                    b.to_vec().try_into().map_err(|_| {
                        MsgPackError::EncodingError(
                            "close remainder to should be 32 byte public key".to_string(),
                        )
                    })
                })
                .transpose()?,
        })
    }
}

impl TryFrom<algo_models::Transaction> for Transaction {
    type Error = MsgPackError;

    fn try_from(tx: algo_models::Transaction) -> Result<Self, Self::Error> {
        match tx {
            algo_models::Transaction::Payment(payment) => {
                let header = payment.header.into();
                let pay_fields = PayTransactionFields {
                    receiver: ByteBuf::from(payment.receiver.to_vec()),
                    amount: payment.amount,
                    close_remainder_to: payment
                        .close_remainder_to
                        .map(|b| ByteBuf::from(b.to_vec())),
                };

                Ok(Self {
                    header,
                    pay_fields: Some(pay_fields),
                    asset_transfer_fields: None,
                })
            }
            algo_models::Transaction::AssetTransfer(asset_transfer) => {
                let header = asset_transfer.header.into();
                let asset_fields = AssetTransferTransactionFields {
                    asset_id: asset_transfer.asset_id,
                    amount: asset_transfer.amount,
                    receiver: ByteBuf::from(asset_transfer.receiver.to_vec()),
                    asset_sender: asset_transfer
                        .asset_sender
                        .map(|b| ByteBuf::from(b.to_vec())),
                    close_remainder_to: asset_transfer
                        .close_remainder_to
                        .map(|b| ByteBuf::from(b.to_vec())),
                };

                Ok(Self {
                    header,
                    pay_fields: None,
                    asset_transfer_fields: Some(asset_fields),
                })
            }
        }
    }
}

impl From<algo_models::TransactionType> for TransactionType {
    fn from(tx: algo_models::TransactionType) -> Self {
        match tx {
            algo_models::TransactionType::Payment => TransactionType::Payment,
            algo_models::TransactionType::AssetTransfer => TransactionType::AssetTransfer,
            algo_models::TransactionType::AssetFreeze => TransactionType::AssetFreeze,
            algo_models::TransactionType::AssetConfig => TransactionType::AssetConfig,
            algo_models::TransactionType::KeyRegistration => TransactionType::KeyRegistration,
            algo_models::TransactionType::ApplicationCall => TransactionType::ApplicationCall,
        }
    }
}

// Each function need to be explicitly renamed for WASM
// and exported for UniFFI

/// Get the transaction type from the encoded transaction.
/// This is particularly useful when decoding a transaction that has a unknow type
#[ffi_func]
pub fn get_encoded_transaction_type(bytes: &[u8]) -> Result<TransactionType, MsgPackError> {
    let decoded = algo_models::Transaction::decode(bytes)?;

    match decoded {
        algo_models::Transaction::Payment(_) => Ok(TransactionType::Payment),
        algo_models::Transaction::AssetTransfer(_) => Ok(TransactionType::AssetTransfer),
    }
}

#[ffi_func]
pub fn encode_transaction(tx: Transaction) -> Result<Vec<u8>, MsgPackError> {
    let ctx: algo_models::Transaction = tx.try_into()?;
    Ok(ctx.encode()?)
}

#[ffi_func]
pub fn decode_transaction(bytes: &[u8]) -> Result<Transaction, MsgPackError> {
    let ctx: algo_models::Transaction = algo_models::Transaction::decode(bytes)?;
    Ok(ctx.try_into()?)
}

#[ffi_func]
pub fn attach_signature(encoded_tx: &[u8], signature: &[u8]) -> Result<Vec<u8>, MsgPackError> {
    let encoded_tx = algo_models::Transaction::decode(encoded_tx)?;
    let signed_tx = algo_models::SignedTransaction {
        transaction: encoded_tx,
        signature: signature.try_into().expect("signature should be 64 bytes"),
    };
    Ok(signed_tx.encode()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_encoded_transaction_type() {
        // Create a minimal payment transaction
        let header = TransactionHeader {
            transaction_type: TransactionType::Payment,
            sender: ByteBuf::from(vec![0; 32]), // 32-byte dummy public key
            fee: 1000,
            first_valid: 1000,
            last_valid: 2000,
            genesis_hash: None,
            genesis_id: None,
            note: None,
            rekey_to: None,
            lease: None,
            group: None,
        };

        let pay_fields = PayTransactionFields {
            receiver: ByteBuf::from(vec![1; 32]), // 32-byte dummy receiver
            amount: 1000000,
            close_remainder_to: None,
        };

        let tx = Transaction {
            header,
            pay_fields: Some(pay_fields),
            asset_transfer_fields: None,
        };

        // Encode the transaction
        let encoded = encode_transaction(tx).unwrap();

        // Test the get_encoded_transaction_type function
        let tx_type = get_encoded_transaction_type(&encoded).unwrap();
        assert_eq!(tx_type, TransactionType::Payment);
    }
}
