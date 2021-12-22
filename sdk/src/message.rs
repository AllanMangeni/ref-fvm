use fvm_shared::{
    econ::TokenAmount,
    encoding::{Cbor, DAG_CBOR},
    error::ExitCode,
    ActorID, MethodNum,
};

use crate::{
    abort,
    error::{IntoSyscallResult, SyscallResult},
    ipld::{BlockId, Codec},
    sys,
};

/// Returns the ID address of the caller.
#[inline(always)]
pub fn caller() -> SyscallResult<ActorID> {
    unsafe { sys::message::caller().into_syscall_result() }
}

/// Returns the ID address of the actor.
#[inline(always)]
pub fn receiver() -> SyscallResult<ActorID> {
    unsafe { sys::message::receiver().into_syscall_result() }
}

/// Returns the message's method number.
#[inline(always)]
pub fn method_number() -> SyscallResult<MethodNum> {
    unsafe { sys::message::method_number().into_syscall_result() }
}

/// Returns the message codec and parameters.
pub fn params_raw(id: BlockId) -> SyscallResult<(Codec, Vec<u8>)> {
    unsafe {
        let (codec, size) = sys::ipld::stat(id).into_syscall_result()?;
        let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
        let ptr = buf.as_mut_ptr();
        let bytes_read = sys::ipld::read(id, 0, ptr, size).into_syscall_result()?;
        debug_assert!(bytes_read == size, "read an unexpected number of bytes");
        Ok((codec, buf))
    }
}

/// Returns the value received from the caller in AttoFIL.
#[inline(always)]
pub fn value_received() -> SyscallResult<TokenAmount> {
    unsafe {
        let (lo, hi) = sys::message::value_received().into_syscall_result()?;
        Ok(TokenAmount::from(hi) << 64 | TokenAmount::from(lo))
    }
}

/// Fetches the input parameters as raw bytes, and decodes them locally
/// into type T using cbor serde. Failing to decode will abort execution.
pub fn params_cbor<T: Cbor>(id: BlockId) -> SyscallResult<T> {
    let (codec, raw) = params_raw(id)?;
    debug_assert!(codec == DAG_CBOR, "parameters codec was not cbor");
    match fvm_shared::encoding::from_slice(raw.as_slice()) {
        Ok(v) => Ok(v),
        Err(e) => abort(
            ExitCode::ErrSerialization as u32,
            Some(format!("could not deserialize parameters as cbor: {:?}", e).as_str()),
        ),
    }
}
