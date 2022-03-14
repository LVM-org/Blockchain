use solana_program::{msg, program_error::ProgramError};
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
}

impl LVMInstruction {
    /// Unpacks a byte buffer into a [LVMInstruction](enum.LVMInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        msg!("Unpack Intructions");
        Ok(match tag {
            0 => Self::CreateMedia {
                price_per_minute: Self::unpack_price_per_minute(rest)?,
                distributor_fee: Self::unpack_distributor_fee(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_price_per_minute(input: &[u8]) -> Result<u64, ProgramError> {
        let price = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        msg!("Unpack price per minutes : {}", price);
        Ok(price)
    }

    fn unpack_distributor_fee(input: &[u8]) -> Result<u64, ProgramError> {
        let fee_percentage = input
            .get(9..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        msg!("Unpack percentage fee minutes : {}", fee_percentage);
        Ok(fee_percentage)
    }
}
