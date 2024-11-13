use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction::create_account, system_program, sysvar::Sysvar
};
use borsh::BorshDeserialize;
use spl_token::instruction::transfer;

use crate::{
    constants::CONTRIBUTE_AMOUNT_OFFSET, 
    state::{
        contributor::Contributor, 
        fundraiser::Fundraiser
    }
};

pub fn contribute(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        contributor,
        contributor_account,
        contributor_ata,
        fundraiser,
        vault,
        token_program,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    // transfer into vault
    let amount = u64::try_from_slice(&instruction_data[..CONTRIBUTE_AMOUNT_OFFSET])?;
    
    if !contributor.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }

    // check program_ids for system_program and token_program
    if !spl_token::check_id(&token_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    if !system_program::check_id(&system_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    let transfer_ix = transfer(
        token_program.key, 
        contributor_ata.key, 
        vault.key, 
        contributor.key, 
        &[], 
        amount
    )?;

    invoke(
        &transfer_ix, 
        &[
            token_program.clone(),
            contributor_ata.clone(),
            vault.clone(),
            contributor.clone()
        ]
    )?;

    let current_time = Clock::get()?.unix_timestamp;

    // increase amount in Fundraiser account
    Fundraiser::increase_amount(fundraiser, amount, current_time)?;

    // increase amount in Contributor account, initializing if necessary
    if contributor.owner == system_program.key && contributor_account.data_is_empty() {

        let (expected_pda, bump) = Pubkey::find_program_address(
            &[
                b"contributor",
                fundraiser.key.as_ref(),
                contributor.key.as_ref(),
            ],
            &crate::ID
        );

        if expected_pda != *contributor_account.key {
            return Err(ProgramError::InvalidSeeds)
        }

        let lamports = Rent::get()?.minimum_balance(Contributor::LEN);

        let create_account_ix = create_account(
            contributor.key, 
            contributor_account.key, 
            lamports, 
            Contributor::LEN as u64, 
            &crate::ID
        );

        invoke_signed(
            &create_account_ix, 
            &[
                contributor.clone(),
                contributor_account.clone(),
                system_program.clone(),
            ], 
            &[&[
                b"contributor",
                fundraiser.key.as_ref(),
                contributor.key.as_ref(),
                &[bump]
            ]]
        )?;

        Contributor::init(contributor_account, amount, bump)?;

        Contributor::increase_amount(contributor_account, amount)?;

    } else {
        Contributor::increase_amount(contributor_account, amount)?;
    }

    Ok(())
}