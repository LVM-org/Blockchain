use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct Media {
    pub author_pubkey: Pubkey,
    pub price_per_minute: f64,
    pub distributor_fee: u64,
    pub nft_token: Pubkey,
    pub nft_account_pubkey: Pubkey,
}

impl Sealed for Media {}

impl Pack for Media {
    const LEN: usize = 112;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Media::LEN];
        let (author_pubkey, price_per_minute, distributor_fee, nft_token, nft_account_pubkey) =
            array_refs![src, 32, 8, 8, 32, 32];

        Ok(Media {
            author_pubkey: Pubkey::new_from_array(*author_pubkey),
            price_per_minute: f64::from_le_bytes(*price_per_minute),
            distributor_fee: u64::from_le_bytes(*distributor_fee),
            nft_token: Pubkey::new_from_array(*nft_token),
            nft_account_pubkey: Pubkey::new_from_array(*nft_account_pubkey),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Media::LEN];
        let (
            author_pubkey_dst,
            price_per_minute_dst,
            distributor_fee_dst,
            nft_token_dst,
            nft_account_pubkey_dst,
        ) = mut_array_refs![dst, 32, 8, 8, 32, 32];

        let Media {
            author_pubkey,
            price_per_minute,
            distributor_fee,
            nft_token,
            nft_account_pubkey,
        } = self;

        author_pubkey_dst.copy_from_slice(author_pubkey.as_ref());
        *price_per_minute_dst = price_per_minute.to_le_bytes();
        *distributor_fee_dst = distributor_fee.to_le_bytes();
        nft_token_dst.copy_from_slice(nft_token.as_ref());
        nft_account_pubkey_dst.copy_from_slice(nft_account_pubkey.as_ref());
    }
}
