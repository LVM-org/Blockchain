use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::Account as TokenAccount;

use crate::{
    error::LVMError,
    instruction::LVMInstruction,
    state::{AccessTime, Media},
};

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
            } => Self::process_create_media(accounts, distributor_fee, price_per_minute),
            LVMInstruction::PurchaseAccessTime { time_in_minute } => {
                Self::process_purchase_time(accounts, time_in_minute)
            }
            LVMInstruction::UpdateAccessTime { access_time } => {
                Self::update_access_time(accounts, access_time)
            }
        }
    }

    fn process_create_media(
        accounts: &[AccountInfo],
        distributor_fee: u64,
        price_per_minute: u64,
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

    fn process_purchase_time(accounts: &[AccountInfo], time_in_minute: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let buyer_main_account = next_account_info(account_info_iter)?;
        let lvm_program_account = next_account_info(account_info_iter)?;
        let media_program_account = next_account_info(account_info_iter)?;
        let author_lvm_token_account = next_account_info(account_info_iter)?;
        let distributor_lvm_token_account = next_account_info(account_info_iter)?;
        let buyer_lvm_token_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        let lvm_token_program = next_account_info(account_info_iter)?;

        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        };

        if !buyer_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        };

        if *author_lvm_token_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        };

        if *distributor_lvm_token_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        };

        if *buyer_lvm_token_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        };

        if !rent.is_exempt(
            lvm_program_account.lamports(),
            lvm_program_account.data_len(),
        ) {
            return Err(LVMError::NotRentExempt.into());
        };

        let media_info = Media::unpack_unchecked(&media_program_account.try_borrow_data()?)?;

        // check buyer's balance

        let buyer_token_account =
            TokenAccount::unpack(&buyer_lvm_token_account.try_borrow_data()?)?;

        let total_time_cost = time_in_minute * &media_info.price_per_minute;

        let buyer_token_balance = buyer_token_account.amount;

        if buyer_token_balance < total_time_cost {
            return Err(LVMError::InsufficientTokenBalance.into());
        }

        // distribute token

        let token_for_distributor = total_time_cost * (media_info.distributor_fee / 100);

        let token_for_author = total_time_cost - token_for_distributor;

        // transfer to distributor
        let transfer_to_distributor = spl_token::instruction::transfer(
            lvm_token_program.key,
            buyer_lvm_token_account.key,
            distributor_lvm_token_account.key,
            buyer_main_account.key,
            &[&buyer_main_account.key],
            token_for_distributor,
        )?;

        invoke(
            &transfer_to_distributor,
            &[
                buyer_lvm_token_account.clone(),
                distributor_lvm_token_account.clone(),
                buyer_main_account.clone(),
                lvm_token_program.clone(),
            ],
        )?;

        // transfer to author
        let transfer_to_author = spl_token::instruction::transfer(
            lvm_token_program.key,
            buyer_lvm_token_account.key,
            author_lvm_token_account.key,
            buyer_main_account.key,
            &[&buyer_main_account.key],
            token_for_author,
        )?;

        invoke(
            &transfer_to_author,
            &[
                buyer_lvm_token_account.clone(),
                author_lvm_token_account.clone(),
                buyer_main_account.clone(),
                lvm_token_program.clone(),
            ],
        )?;

        // save read time data

        let mut access_time_info =
            AccessTime::unpack_unchecked(&lvm_program_account.try_borrow_data()?)?;

        access_time_info.owner_pubkey = *buyer_main_account.key;
        access_time_info.total_time = time_in_minute;
        access_time_info.time_spent = 0;

        // save data to account
        AccessTime::pack(
            access_time_info,
            &mut lvm_program_account.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

    fn update_access_time(accounts: &[AccountInfo], access_time: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let lvm_program_account = next_account_info(account_info_iter)?;

        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        };

        let mut access_time_info =
            AccessTime::unpack_unchecked(&lvm_program_account.try_borrow_data()?)?;

        if access_time_info.time_spent > access_time {
            return Err(LVMError::AccessTimeCannotReduce.into());
        };

        access_time_info.time_spent = access_time;

        if access_time >= access_time_info.time_spent {
            // close access time account
            **payer_account.try_borrow_mut_lamports()? = payer_account
                .lamports()
                .checked_add(lvm_program_account.lamports())
                .ok_or(LVMError::AmountOverflow)?;
            **lvm_program_account.try_borrow_mut_lamports()? = 0;
            *lvm_program_account.try_borrow_mut_data()? = &mut []
        } else {
            // save data to account
            AccessTime::pack(
                access_time_info,
                &mut lvm_program_account.try_borrow_mut_data()?,
            )?;
        };

        Ok(())
    }
}
