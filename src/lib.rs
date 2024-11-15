mod state;
mod instructions;
mod constants;
mod error;
mod tests;

use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    entrypoint,
    program_error::ProgramError, 
    pubkey::Pubkey,
    pubkey
};

use instructions::{
    FundraiserInstructions,
    initialize::initialize,
    contribute::contribute,
    refund::refund,
    checker::checker,
};

pub const ID: Pubkey =
    pubkey!("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let (discriminator, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => initialize(accounts, instruction_data),
        FundraiserInstructions::Contribute => contribute(accounts, instruction_data),
        FundraiserInstructions::Refund => refund(accounts),
        FundraiserInstructions::Checker => checker(accounts),
    }
}
