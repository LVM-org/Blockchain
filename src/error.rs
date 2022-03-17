use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum LVMError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Insufficient Token Balance
    #[error("Insufficient Token Balance")]
    InsufficientTokenBalance,
    /// Access Time Cannot Be Reduce
    #[error("Access Time Cannot Be Reduce")]
    AccessTimeCannotReduce,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
}

impl From<LVMError> for ProgramError {
    fn from(e: LVMError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
