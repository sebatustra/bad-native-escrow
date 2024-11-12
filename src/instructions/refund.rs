use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program::invoke_signed, 
    program_error::ProgramError, 
    pubkey::Pubkey
};
use spl_token::instruction::transfer;
use crate::state::contributor::Contributor;


pub fn refund(
    accounts: &[AccountInfo],
) -> ProgramResult {

    let [
        contributor,
        contributor_account,
        contributor_ata,
        fundraiser,
        vault,
        token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    let seeds = &[
        b"contributor",
        fundraiser.key.as_ref(),
        contributor.key.as_ref(),
    ];

    let (
        contributor_account_pda, 
        contributor_account_bump
    ) = Pubkey::find_program_address(
        seeds,
        &crate::ID
    );

    assert_eq!(contributor_account_pda, *contributor_account.key);

    let contributor_account_data = Contributor::try_from_slice(
        &contributor_account.try_borrow_data()?
    )?;

    // we transfer from the vault back to the contributor the amount they have in their account

    let transfer_ix = transfer(
        token_program.key, 
        vault.key, 
        contributor_ata.key, 
        fundraiser.key, 
        &[], 
        contributor_account_data.amount
    )?;

    let seeds = &[
        b"contributor".as_ref(),
        fundraiser.key.as_ref(),
        contributor.key.as_ref(),
        &[contributor_account_bump]
    ];

    invoke_signed(
        &transfer_ix, 
        &[
            token_program.clone(),
            vault.clone(),
            contributor_ata.clone(),
            fundraiser.clone(),
        ], 
        &[seeds]
    )?;

    // we close the contributor_account
    let balance = contributor_account.lamports();
    contributor_account.realloc(0, false)?;
    **contributor_account.lamports.borrow_mut() = 0;
    **contributor.lamports.borrow_mut() += balance;
    contributor_account.assign(&Pubkey::default());

    Ok(())
}