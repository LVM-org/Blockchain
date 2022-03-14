use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{error::LVMError, instruction::LVMInstruction, state::Media};

pub struct Processor;
impl Processor {
    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LVMInstruction::unpack(instruction_data)?;

        match instruction {
            LVMInstruction::CreateMedia {
                distributor_fee,
                price_per_minute,
            } => {
                msg!("Instruction: Create Media");
                Self::process_create_media(accounts, distributor_fee, price_per_minute)
            }
        }
    }

    fn process_create_media(
        accounts: &[AccountInfo],
        distributor_fee: u64,
        price_per_minute: f64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let author = next_account_info(account_info_iter)?;

        if !author.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        };

        let nft_token_account = next_account_info(account_info_iter)?;

        if *nft_token_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let lvm_account = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(lvm_account.lamports(), lvm_account.data_len()) {
            return Err(LVMError::NotRentExempt.into());
        };

        let mut media_info = Media::unpack_unchecked(&lvm_account.try_borrow_data()?)?;

        let nft_program = next_account_info(account_info_iter)?;

        media_info.author_pubkey = *author.key;
        media_info.distributor_fee = distributor_fee;
        media_info.nft_account_pubkey = *nft_token_account.key;
        media_info.price_per_minute = price_per_minute;
        media_info.nft_token = *nft_program.key;

        // save data to account
        Media::pack(media_info, &mut lvm_account.try_borrow_mut_data()?)?;

        Ok(())
    }
}
