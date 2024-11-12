
use solana_program::{
    account_info::AccountInfo, 
    pubkey::Pubkey, 
    entrypoint::ProgramResult
};
use borsh::{BorshDeserialize, BorshSerialize};

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

        let mut fundraiser_account = Self::try_from_slice(&fundraiser.try_borrow_mut_data()?)?;

        fundraiser_account.clone_from(&Self {
            maker: *maker,
            mint_to_raise: *mint_to_raise,
            amount_to_raise,
            current_amount,
            time_started,
            duration,
            bump
        });

        fundraiser_account.serialize(&mut *fundraiser.data.borrow_mut())?;

        Ok(())
    }

    #[inline]
    pub fn increase_amount(
        fundraiser: &AccountInfo,
        amount_to_increase: u64
    ) -> ProgramResult {
        let mut fundraiser_account = Self::try_from_slice(&fundraiser.try_borrow_mut_data()?)?;

        fundraiser_account.current_amount += amount_to_increase;

        fundraiser_account.serialize(&mut *fundraiser.data.borrow_mut())?;
        
        Ok(())
    }
}