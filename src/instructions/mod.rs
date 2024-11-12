use solana_program::program_error::ProgramError;

pub mod initialize;
pub mod checker;
pub mod contribute;
pub mod refund;

#[derive(Copy, Clone, Debug)]
pub enum FundraiserInstructions {
    Initialize,
    Contribute,
    Refund,
    Checker,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::Contribute),
            2 => Ok(FundraiserInstructions::Refund),
            3 => Ok(FundraiserInstructions::Checker),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}