use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar
};
use spl_token::instruction::transfer;
use crate::{constants::SECONDS_TO_DAYS, error::FundraiserError, state::{contributor::Contributor, fundraiser::Fundraiser}};

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

    //we verify the token program id
    if !spl_token::check_id(&token_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    // we verify that contributor account is owned by this program
    if contributor_account.owner != &crate::ID {
        return Err(ProgramError::InvalidAccountOwner)
    }

    //we verify that the contributor account has data
    if contributor_account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount)
    }

    // we check contributor is signer
    if !contributor.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }
    
    // we verify the contributor_account corresponds to the contributor
    let (contributor_pda, _) = Pubkey::find_program_address(
        &[
            b"contributor",
            fundraiser.key.as_ref(),
            contributor.key.as_ref(),
        ], 
        &crate::ID
    );

    if contributor_pda != *contributor_account.key {
        return Err(ProgramError::InvalidAccountData)
    }

    let contributor_account_data = Contributor::try_from_slice(
        &contributor_account.try_borrow_data()?
    )?;

    let mut fundraiser_account = Fundraiser::try_from_slice(
        &fundraiser.try_borrow_mut_data()?
    )?;

    let current_time = Clock::get()?.unix_timestamp;
    
    // we verify that the fundraiser has ended
    if fundraiser_account.duration <= ((current_time - fundraiser_account.time_started) / SECONDS_TO_DAYS) as u8 {
        return Err(FundraiserError::FundraiserNotEnded.into())
    }

    // we transfer from the vault back to the contributor the amount they have in their account
    let transfer_ix = transfer(
        token_program.key, 
        vault.key, 
        contributor_ata.key, 
        fundraiser.key, 
        &[], 
        contributor_account_data.amount
    )?;

    invoke_signed(
        &transfer_ix, 
        &[
            token_program.clone(),
            vault.clone(),
            contributor_ata.clone(),
            fundraiser.clone(),
        ], 
        &[&[
            b"contributor".as_ref(),
            fundraiser.key.as_ref(),
            contributor.key.as_ref(),
            &[contributor_account_data.bump]
        ]]
    )?;

    fundraiser_account.current_amount -= contributor_account_data.amount;

    // we close the contributor_account
    let balance = contributor_account.lamports();
    contributor_account.realloc(0, false)?;
    **contributor_account.lamports.borrow_mut() = 0;
    **contributor.lamports.borrow_mut() += balance;
    contributor_account.assign(&Pubkey::default());

    Ok(())
}