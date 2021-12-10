use bls_signatures::{
    verify_messages, PublicKey as BlsPubKey, Serialize, Signature as BlsSignature,
};
use fvm_shared::address::{Address, Protocol};
use fvm_shared::crypto::signature::{Error, Signature, SECP_SIG_LEN};
use fvm_shared::encoding::blake2b_256;
use libsecp256k1::Error as SecpError;
use libsecp256k1::{recover, Message, RecoveryId, Signature as EcsdaSignature};
use std::error;

/// Checks if a signature is valid given data and address.
pub fn verify(sign: &Signature, data: &[u8], addr: &Address) -> Result<(), String> {
    match addr.protocol() {
        Protocol::BLS => verify_bls_sig(sign.bytes(), data, addr),
        Protocol::Secp256k1 => verify_secp256k1_sig(sign.bytes(), data, addr),
        _ => Err("Address must be resolved to verify a signature".to_owned()),
    }
}

/// Returns `String` error if a bls signature is invalid.
pub(crate) fn verify_bls_sig(signature: &[u8], data: &[u8], addr: &Address) -> Result<(), String> {
    let pub_k = addr.payload_bytes();

    // generate public key object from bytes
    let pk = BlsPubKey::from_bytes(&pub_k).map_err(|e| e.to_string())?;

    // generate signature struct from bytes
    let sig = BlsSignature::from_bytes(signature).map_err(|e| e.to_string())?;

    // BLS verify hash against key
    if verify_messages(&sig, &[data], &[pk]) {
        Ok(())
    } else {
        Err(format!(
            "bls signature verification failed for addr: {}",
            addr
        ))
    }
}

/// Returns `String` error if a secp256k1 signature is invalid.
fn verify_secp256k1_sig(signature: &[u8], data: &[u8], addr: &Address) -> Result<(), String> {
    if signature.len() != SECP_SIG_LEN {
        return Err(format!(
            "Invalid Secp256k1 signature length. Was {}, must be 65",
            signature.len()
        ));
    }

    // blake2b 256 hash
    let hash = blake2b_256(data);

    // Ecrecover with hash and signature
    let mut sig = [0u8; SECP_SIG_LEN];
    sig[..].copy_from_slice(signature);
    let rec_addr = ecrecover(&hash, &sig).map_err(|e| e.to_string())?;

    // check address against recovered address
    if &rec_addr == addr {
        Ok(())
    } else {
        Err("Secp signature verification failed".to_owned())
    }
}
/// Aggregates and verifies bls signatures collectively.
pub fn verify_bls_aggregate(data: &[&[u8]], pub_keys: &[&[u8]], aggregate_sig: &Signature) -> bool {
    // If the number of public keys and data does not match, then return false
    if data.len() != pub_keys.len() {
        return false;
    }
    if data.is_empty() {
        return true;
    }

    let sig = match BlsSignature::from_bytes(aggregate_sig.bytes()) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let pk_map_results: Result<Vec<_>, _> =
        pub_keys.iter().map(|x| BlsPubKey::from_bytes(x)).collect();

    let pks = match pk_map_results {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Does the aggregate verification
    verify_messages(&sig, data, &pks[..])
}

/// Return Address for a message given it's signing bytes hash and signature.
pub fn ecrecover(hash: &[u8; 32], signature: &[u8; SECP_SIG_LEN]) -> Result<Address, Error> {
    // generate types to recover key from
    let rec_id = RecoveryId::parse(signature[64])?;
    let message = Message::parse(hash);

    // Signature value without recovery byte
    let mut s = [0u8; 64];
    s.clone_from_slice(signature[..64].as_ref());
    // generate Signature
    let sig = EcsdaSignature::parse_standard(&s)?;

    let key = recover(&message, &sig, &rec_id)?;
    let ret = key.serialize();
    let addr = Address::new_secp256k1(&ret)?;
    Ok(addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bls_signatures::{PrivateKey, PublicKey, Serialize, Signature as BlsSignature};
    use libsecp256k1::{sign, PublicKey, SecretKey};
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn bls_agg_verify() {
        // The number of signatures in aggregate
        let num_sigs = 10;
        let message_length = num_sigs * 64;

        let rng = &mut ChaCha8Rng::seed_from_u64(11);

        let msg = (0..message_length).map(|_| rng.gen()).collect::<Vec<u8>>();
        let data: Vec<&[u8]> = (0..num_sigs).map(|x| &msg[x * 64..(x + 1) * 64]).collect();

        let private_keys: Vec<PrivateKey> =
            (0..num_sigs).map(|_| PrivateKey::generate(rng)).collect();
        let public_keys: Vec<_> = private_keys
            .iter()
            .map(|x| x.public_key().as_bytes())
            .collect();

        let signatures: Vec<BlsSignature> = (0..num_sigs)
            .map(|x| private_keys[x].sign(data[x]))
            .collect();

        let mut public_keys_slice: Vec<&[u8]> = vec![];
        for i in 0..num_sigs {
            public_keys_slice.push(&public_keys[i]);
        }

        let calculated_bls_agg =
            Signature::new_bls(bls_signatures::aggregate(&signatures).unwrap().as_bytes());
        assert_eq!(
            verify_bls_aggregate(&data, &public_keys_slice, &calculated_bls_agg),
            true
        );
    }

    #[test]
    fn secp_ecrecover() {
        let rng = &mut ChaCha8Rng::seed_from_u64(8);

        let priv_key = SecretKey::random(rng);
        let pub_key = PublicKey::from_secret_key(&priv_key);
        let secp_addr = Address::new_secp256k1(&pub_key.serialize()).unwrap();

        let hash = blake2b_256(&[8, 8]);
        let msg = Message::parse(&hash);

        // Generate signature
        let (sig, recovery_id) = sign(&msg, &priv_key);
        let mut signature = [0; 65];
        signature[..64].copy_from_slice(&sig.serialize());
        signature[64] = recovery_id.serialize();

        assert_eq!(ecrecover(&hash, &signature).unwrap(), secp_addr);
    }
}

/// Crypto error
#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// Failed to produce a signature
    #[error("Failed to sign data {0}")]
    SigningError(String),
    /// Unable to perform ecrecover with the given params
    #[error("Could not recover public key from signature: {0}")]
    InvalidRecovery(String),
    /// Provided public key is not understood
    #[error("Invalid generated pub key to create address: {0}")]
    InvalidPubKey(#[from] AddressError),
}

impl From<Box<dyn error::Error>> for Error {
    fn from(err: Box<dyn error::Error>) -> Error {
        // Pass error encountered in signer trait as module error type
        Error::SigningError(err.to_string())
    }
}

impl From<SecpError> for Error {
    fn from(err: SecpError) -> Error {
        match err {
            SecpError::InvalidRecoveryId => Error::InvalidRecovery(format!("{:?}", err)),
            _ => Error::SigningError(format!("{:?}", err)),
        }
    }
}

impl From<EncodingError> for Error {
    fn from(err: EncodingError) -> Error {
        // Pass error encountered in signer trait as module error type
        Error::SigningError(err.to_string())
    }
}
