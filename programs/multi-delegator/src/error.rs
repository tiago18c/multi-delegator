use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient amount")]
    InsufficientAmount,
    #[msg("Invalid state")]
    InvalidState,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid transfer - check of delegation kind and amount make sense")]
    InvalidTransfer,
    #[msg("Invalid duration")]
    InvalidDuration,
    #[msg("Invalid kind")]
    InvalidKind,
    #[msg("Invalid destination account")]
    InvalidDestination,
}
