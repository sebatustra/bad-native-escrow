
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey
};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{constants::SECONDS_TO_DAYS, error::FundraiserError};

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = 96;

    #[inline]
    pub fn init(
        fundraiser: &AccountInfo,
        maker: &Pubkey,
        mint_to_raise: &Pubkey,
        amount_to_raise: u64,
        current_amount: u64,
        time_started: i64,
        duration: u8,
        bump: u8
    ) -> ProgramResult {
        
        let fundraiser_data = Fundraiser {
            maker: *maker,
            mint_to_raise: *mint_to_raise,
            amount_to_raise,
            current_amount,
            time_started,
            duration,
            bump
        };
        
        // Serialize the new instance directly into the account data
        fundraiser_data.serialize(&mut *fundraiser.data.borrow_mut())?;

        Ok(())
    }

    #[inline]
    pub fn increase_amount(
        fundraiser: &AccountInfo,
        amount_to_increase: u64,
        current_time: i64
    ) -> ProgramResult {
        let mut fundraiser_account = Self::try_from_slice(&fundraiser.try_borrow_mut_data()?)?;

        if fundraiser_account.duration > ((current_time - fundraiser_account.time_started) / SECONDS_TO_DAYS) as u8 {
            return Err(FundraiserError::FundraiserEnded.into())
        }

        fundraiser_account.current_amount += amount_to_increase;

        fundraiser_account.serialize(&mut *fundraiser.data.borrow_mut())?;
        
        Ok(())
    }
}