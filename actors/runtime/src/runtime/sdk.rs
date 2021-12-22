use std::convert::TryFrom;

use blockstore::{Block, Blockstore};
use cid::{multihash::Code, Cid};
use fvm_sdk::ipld;

use crate::{actor_error, ActorError};

/// A blockstore suitable for use within actors.
pub struct ActorBlockstore;

impl Blockstore for ActorBlockstore {
    type Error = ActorError;

    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, Self::Error> {
        // If this fails, the _CID_ is invalid. I.e., we have a bug.
        ipld::get(cid)
            .map(Some)
            .map_err(|c| actor_error!(ErrIllegalState; "get failed with {:?} on CID '{}'", c, cid))
    }

    fn put<D>(&self, code: Code, block: &Block<D>) -> Result<Cid, Self::Error>
    where
        D: AsRef<[u8]>,
    {
        // TODO: Don't hard-code the size. Unfortunately, there's no good way to get it from the
        // codec at the moment.
        const SIZE: u32 = 32;
        ipld::put(code.into(), SIZE, block.codec, block.data.as_ref())
            .map_err(|c| actor_error!(ErrIllegalState; "put failed with {:?}", c))
    }

    fn put_keyed(&self, k: &Cid, block: &[u8]) -> Result<(), Self::Error> {
        let k2 = self.put(
            Code::try_from(k.hash().code())
                .map_err(|e| actor_error!(ErrSerialization, e.to_string()))?,
            &Block::new(k.codec(), block),
        )?;
        if k != &k2 {
            Err(actor_error!(ErrSerialization; "put block with cid {} but has cid {}", k, k2))
        } else {
            Ok(())
        }
    }
}
