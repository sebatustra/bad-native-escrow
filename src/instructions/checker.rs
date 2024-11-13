use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program::invoke_signed, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey
};
use crate::{error::FundraiserError, state::fundraiser::Fundraiser};
use spl_token::instruction::{
    transfer, 
    close_account
};


pub fn checker(
    accounts: &[AccountInfo]
) -> ProgramResult {
    // we verify that the target amount has been reached

    let [
        maker,
        maker_ata,
        fundraiser,
        vault,
        token_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    // check if the token_program had the correct id
    if !spl_token::check_id(&token_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    // we check the maker is signer
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }

    let fundraiser_account = Fundraiser::try_from_slice(
        &fundraiser.try_borrow_data()?
    )?;

    if fundraiser_account.maker != *maker.key {
        return Err(FundraiserError::InvalidFundraiserMaker.into())
    }

    let vault_balance = spl_token::state::Account::unpack(
        &vault.try_borrow_data()?
    )?.amount;


    if fundraiser_account.amount_to_raise < vault_balance {
        return Err(FundraiserError::AmountRaisedNotEnough.into())
    }

    let transfer_ix = transfer(
        token_program.key, 
        vault.key, 
        maker_ata.key, 
        fundraiser.key, 
        &[], 
        vault_balance
    )?;

    let seeds = &[
        b"fundraiser",
        maker.key.as_ref(),
        &[fundraiser_account.bump]
    ];

    invoke_signed(
        &transfer_ix, 
        &[
            token_program.clone(),
            vault.clone(),
            maker_ata.clone(),
            fundraiser.clone()
        ], 
        &[seeds]
    )?;

    // we close the fundraiser account and vault
    let close_vault_ix = close_account(
        token_program.key, 
        vault.key, 
        maker.key, 
        fundraiser.key, 
        &[]
    )?;

    invoke_signed(
        &close_vault_ix, 
        &[
            token_program.clone(),
            vault.clone(),
            maker.clone(),
            fundraiser.clone()
        ], 
        &[seeds]
    )?;

    let balance = fundraiser.lamports();
    fundraiser.realloc(0, false)?;
    **fundraiser.lamports.borrow_mut() = 0;
    **maker.lamports.borrow_mut() += balance;
    fundraiser.assign(&Pubkey::default());
    
    Ok(())
}