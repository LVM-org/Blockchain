use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::LVMError::InvalidInstruction;

pub enum LVMInstruction {
    ///
    ///
    /// Create Media:
    ///
    /// 0. `[signer]` The system account of the author
    /// 1. `[]` NFT token associated account that should be owned by the author
    /// 3. `[writable]` The LVM program account that would hold the NFT media data.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The NFT program
    CreateMedia {
        /// The price per minutes in LVM token
        price_per_minute: u64,
        /// The sales percentage fee (1 - 100) given to distibutors for access time sales
        distributor_fee: u64,
    },
    ///
    ///
    /// Purchase Access Time:
    ///
    /// 0. `[signer]` The payer system account
    /// 1. `[signer, writable]` Buyer main account
    /// 2. `[writable]` The LVM program account that would hold the purchased time data
    /// 3. `[]` Media Program account
    /// 4. `[writable]` Author LVM token associated account.
    /// 5. `[writable]` Distributor LVM token associated account
    /// 6. `[writable]` Buyer LVM token associated account
    /// 7. `[]` The rent sysvar
    /// 8. `[]` The LVM token
    PurchaseAccessTime {
        /// Total time in minutes to purchase
        time_in_minute: u64,
    },
    ///
    ///
    /// Update Access Time:
    ///
    /// 0. `[signer]` The payer system account
    /// 1. `[writable]` The LVM program account that would hold the purchased time data
    /// 2. `[]` The rent sysvar
    UpdateAccessTime {
        /// present access time
        access_time: u64,
    },
}

impl LVMInstruction {
    /// Unpacks a byte buffer into a [LVMInstruction](enum.LVMInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateMedia {
                price_per_minute: Self::unpack_integer(&rest, &1)?,
                distributor_fee: Self::unpack_integer(&rest, &2)?,
            },
            1 => Self::PurchaseAccessTime {
                time_in_minute: Self::unpack_integer(&rest, &1)?,
            },
            2 => Self::UpdateAccessTime {
                access_time: Self::unpack_integer(&rest, &1)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_integer(input: &[u8], position: &u32) -> Result<u64, ProgramError> {
        let data;

        match position {
            1 => {
                data = input.get(..8);
            }
            2 => {
                data = input.get(8..16);
            }
            _ => return Err(InvalidInstruction.into()),
        }

        let final_integer = data
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(final_integer)
    }
}
