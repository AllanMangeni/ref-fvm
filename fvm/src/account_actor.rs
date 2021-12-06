//! This module contains the minimal logic for the FVM to handle account actor
//! auto-creation (on first transfer). This coupling between the FVM and a
//! concrete actor must eventually go. (TODO)

use crate::state_tree::ActorState;
use cid::{multihash::Code, Cid};
use fvm_shared::address::Address;

use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::encoding::{to_vec, DAG_CBOR};
use lazy_static::lazy_static;

// TODO: This shouldn't be defined here.
const IPLD_RAW: u64 = 0x55;

lazy_static!(
    pub static ref SYSTEM_ACTOR_ADDR: Address = Address::new_id(0);

    /// Cid of the empty array Cbor bytes (`EMPTY_ARR_BYTES`).
    pub static ref EMPTY_ARR_CID: Cid = {
        let empty = to_vec::<[(); 0]>(&[]).unwrap();
        Cid::new_v1(DAG_CBOR, Code::Blake2b256.digest(empty))
    };

    pub static ref ZERO_STATE: ActorState = ActorState {
        code: Cid::new_v1(IPLD_RAW, Code::Identity.digest(b"fil/5/account")),
        state: *EMPTY_ARR_CID,
        sequence: 0,
        balance: TokenAmount::zero(),
    };
);
