use anchor_lang::prelude::*;

declare_id!("9GjJt2mJXY9szY7TyfKqBpbb8Nc95vgjnRenXZWX17xH");

mod instructions;
mod error;
mod state;
pub use instructions::*;
pub use state::*;

#[program]
pub mod multi_delegator {

    use super::*;


    pub fn add_delegate(ctx: Context<AddDelegate>, kind: DelegationKind) -> Result<()> {
        instructions::add_delegate::handler(ctx, kind)
    }

    pub fn revoke_delegate(ctx: Context<RevokeDelegate>) -> Result<()> {
        instructions::revoke_delegate::handler(ctx)
    }

    pub fn accept_delegate(ctx: Context<AcceptDelegate>) -> Result<()> {
        instructions::accept_delegate::handler(ctx)
    }   

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        instructions::transfer::handler(ctx, amount)
    }

    pub fn transfer_recurring(ctx: Context<TransferRecurring>) -> Result<()> {
        instructions::transfer_recurring::handler(ctx)
    }
}

