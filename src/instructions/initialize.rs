use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program::{invoke, invoke_signed}, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    rent::Rent, 
    system_instruction::create_account, 
    system_program, 
    sysvar::Sysvar
};
use spl_token::instruction::initialize_account;
use crate::{
    constants::{
        AMOUNT_TO_RAISE_OFFSET, 
        DURATION_OFFSET, 
        TIME_STARTED_OFFSET
    }, 
    state::fundraiser::Fundraiser
};

pub fn initialize(
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        system_program,
        token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    let amount_to_raise = u64::try_from_slice(&instruction_data[..AMOUNT_TO_RAISE_OFFSET])?;
    let time_started = i64::try_from_slice(&instruction_data[AMOUNT_TO_RAISE_OFFSET..TIME_STARTED_OFFSET])?;
    let duration = u8::try_from_slice(&instruction_data[TIME_STARTED_OFFSET..DURATION_OFFSET])?;


    let (fundraiser_pda, bump) = Pubkey::find_program_address(
        &[b"fundraiser", maker.key.as_ref()], 
        &crate::ID
    );
    
    // Check that maker is signing
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }

    // check we have the correct pda for fundraiser
    if &fundraiser_pda != fundraiser.key {
        return Err(ProgramError::InvalidSeeds)
    }

    // check the fundraiser owner is system program since it has not been initialized
    if *fundraiser.owner != system_program::ID {
        return Err(ProgramError::InvalidAccountOwner)
    }

    // we check that data is empty
    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized)
    }

    // we check token_program id is correct
    if !spl_token::check_id(token_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    // we check the id of system_program
    if !system_program::check_id(&system_program.key) {
        return Err(ProgramError::IncorrectProgramId)
    }

    let rent = Rent::get()?;

    let minimum_balance = rent.minimum_balance(Fundraiser::LEN);

    let init_ix = create_account(
        maker.key, 
        fundraiser.key, 
        minimum_balance, 
        Fundraiser::LEN as u64, 
        &crate::ID
    );

    invoke_signed(
        &init_ix, 
        &[maker.clone(), fundraiser.clone()], 
        &[&[b"fundraiser", maker.key.as_ref(), &[bump]]]
    )?;

    Fundraiser::init(
        fundraiser, 
        maker.key, 
        mint_to_raise.key, 
        amount_to_raise, 
        0, 
        time_started, 
        duration, 
        bump
    )?;

    let vault_len = spl_token::state::Account::LEN;
    let vault_minimum_balance = rent.minimum_balance(vault_len);

    let create_vault_ix = create_account(
        maker.key, 
        vault.key, 
        vault_minimum_balance, 
        vault_len as u64, 
        token_program.key
    );

    invoke(&create_vault_ix, &[maker.clone(), vault.clone()])?;

    let initialize_vault_ix = initialize_account(
        token_program.key, 
        vault.key, 
        mint_to_raise.key, 
        fundraiser.key
    )?;

    invoke_signed(
        &initialize_vault_ix, 
        &[
            vault.clone(),
            mint_to_raise.clone(),
            fundraiser.clone(),
            token_program.clone(),
            system_program.clone()
        ], 
        &[&[b"fundraiser", maker.key.as_ref(), &[bump]]]
    )?;
    
    Ok(())
}