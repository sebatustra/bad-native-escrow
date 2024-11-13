use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct Contributor {
    pub amount: u64,
    pub bump: u8,
}

impl Contributor {
    pub const LEN: usize = 9;

    #[inline]
    pub fn init(
        contributor_pda: &AccountInfo,
        amount: u64,
        bump: u8
    ) -> ProgramResult {
        let mut contributor_account = Self::try_from_slice(&contributor_pda.try_borrow_mut_data()?)?;

        contributor_account.amount = amount;
        contributor_account.bump = bump;
        
        contributor_account.serialize(&mut *contributor_pda.data.borrow_mut())?;

        Ok(())
    }

    pub fn increase_amount(
        contributor_pda: &AccountInfo,
        amount_to_increase: u64
    ) -> ProgramResult {

        let mut contributor_account = Self::try_from_slice(&contributor_pda.try_borrow_mut_data()?)?;

        contributor_account.amount += amount_to_increase;

        contributor_account.serialize(&mut *contributor_pda.data.borrow_mut())?;

        Ok(())
    }


}