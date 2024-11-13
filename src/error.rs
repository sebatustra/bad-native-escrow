use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum FundraiserError {
    #[error("The Fundraiser had ended!")]
    FundraiserEnded,
    #[error("The fundraiser has not ended yet")]
    FundraiserNotEnded,
    #[error("The fundraiser maker is invalid")]
    InvalidFundraiserMaker,
    #[error("The amount raised is not enough")]
    AmountRaisedNotEnough,
}

impl From<FundraiserError> for ProgramError {
    fn from(error: FundraiserError) -> Self {
        ProgramError::Custom(error as u32)
    }
}