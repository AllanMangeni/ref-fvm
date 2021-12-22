use cid::Cid;
use fvm_shared::{address::Address, econ::TokenAmount};

use crate::{
    error::{IntoSyscallResult, SyscallResult},
    sys, MAX_CID_LEN,
};

/// Get the IPLD root CID.
pub fn get_root() -> SyscallResult<Cid> {
    // I really hate this CID interface. Why can't I just have bytes?
    let mut buf = [0u8; MAX_CID_LEN];
    unsafe {
        let len = sys::sself::get_root(buf.as_mut_ptr(), buf.len() as u32).into_syscall_result()?
            as usize;
        if len > buf.len() {
            // TODO: re-try with a larger buffer?
            panic!("CID too big: {} > {}", len, buf.len())
        }
        Ok(Cid::read_bytes(&buf[..len]).expect("runtime returned an invalid CID"))
    }
}

/// Set the actor's state-tree root. The CID must be in the "reachable" set.
pub fn set_root(cid: &Cid) -> SyscallResult<()> {
    let mut buf = [0u8; MAX_CID_LEN];
    cid.write_bytes(&mut buf[..])
        .expect("CID encoding should not fail");
    unsafe { sys::sself::set_root(buf.as_ptr()).into_syscall_result() }
}

/// Gets the current balance for the calling actor.
#[inline(always)]
pub fn current_balance() -> SyscallResult<TokenAmount> {
    unsafe {
        let (lo, hi) = sys::sself::current_balance().into_syscall_result()?;
        Ok(TokenAmount::from(hi) << 64 | TokenAmount::from(lo))
    }
}

/// Destroys the calling actor, sending its current balance
/// to the supplied address, which cannot be itself.
pub fn self_destruct(beneficiary: Address) -> SyscallResult<()> {
    let bytes = beneficiary.to_bytes();
    unsafe { sys::sself::self_destruct(bytes.as_ptr(), bytes.len() as u32).into_syscall_result() }
}
