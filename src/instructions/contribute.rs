use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program::{invoke, invoke_signed}, 
    program_error::ProgramError, 
    pubkey::Pubkey, 
    rent::Rent, 
    system_instruction::create_account, 
    sysvar::Sysvar
};
use borsh::BorshDeserialize;
use spl_token::instruction::transfer;

use crate::{
    constants::{
        CONTRIBUTE_AMOUNT_OFFSET, 
        CONTRIBUTE_BUMP_OFFSET
    }, 
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
    let bump = u8::try_from_slice(&instruction_data[CONTRIBUTE_AMOUNT_OFFSET..CONTRIBUTE_BUMP_OFFSET])?;
    

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

    // increase amount in Fundraiser account
    Fundraiser::increase_amount(fundraiser, amount)?;

    // increase amount in Contributor account

    if contributor_account.lamports() == 0 && contributor_account.data_is_empty() {
        // we create the account
        let pda_seeds = &[
            b"contributor",
            fundraiser.key.as_ref(),
            contributor.key.as_ref(),
            &[bump]
        ];

        let (expected_pda, _) = Pubkey::find_program_address(
            &pda_seeds[..],
            &crate::ID
        );

        assert_eq!(expected_pda, *contributor_account.key);

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
            &[pda_seeds]
        )?;

        Contributor::init(contributor_account, amount)?;

        Contributor::increase_amount(contributor_account, amount)?;

    } else {
        Contributor::increase_amount(contributor_account, amount)?;
    }

    Ok(())
}