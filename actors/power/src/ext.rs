use cid::Cid;
use fvm_shared::{
    address::Address,
    encoding::{serde_bytes, tuple::*, BytesDe, RawBytes},
    sector::{RegisteredPoStProof, SectorNumber},
};

pub mod init {
    use super::*;

    pub const EXEC_METHOD: u64 = 2;

    /// Init actor Exec Params
    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct ExecParams {
        pub code_cid: Cid,
        pub constructor_params: RawBytes,
    }

    /// Init actor Exec Return value
    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct ExecReturn {
        /// ID based address for created actor
        pub id_address: Address,
        /// Reorg safe address for actor
        pub robust_address: Address,
    }
}

pub mod miner {
    use super::*;

    pub const CONFIRM_SECTOR_PROOFS_VALID_METHOD: u64 = 17;
    pub const ON_DEFERRED_CRON_EVENT_METHOD: u64 = 12;

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct ConfirmSectorProofsParams {
        pub sectors: Vec<SectorNumber>,
    }

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct MinerConstructorParams {
        pub owner: Address,
        pub worker: Address,
        pub control_addresses: Vec<Address>,
        pub window_post_proof_type: RegisteredPoStProof,
        #[serde(with = "serde_bytes")]
        pub peer_id: Vec<u8>,
        pub multi_addresses: Vec<BytesDe>,
    }
}

pub mod reward {
    pub const UPDATE_NETWORK_KPI: u64 = 4;
}
