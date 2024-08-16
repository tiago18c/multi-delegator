use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use crate::state::{Delegation, State};
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct AcceptDelegate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, 
        constraint = delegation.state == State::Pending @ ErrorCode::InvalidState,
        seeds = [b"delegation", token_account.key().as_ref(), delegation.delegate.as_ref(),  &[Into::<u8>::into(&delegation.kind)] ],
        bump = delegation.bump
    )]
    pub delegation: Account<'info, Delegation>,
    
    /// CHECK: just the auth
    #[account(seeds = [b"multi_delegate", token_account.key().as_ref()], bump)]
    pub multi_delegate_authority: UncheckedAccount<'info>,
    
    #[account(mut, token::authority = authority)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<AcceptDelegate>) -> Result<()> {
    let delegation = &mut ctx.accounts.delegation;
    delegation.state = State::Active;

    Delegation::set_delegate_if_needed(
        &ctx.accounts.token_account, 
        ctx.accounts.authority.to_account_info(), 
        ctx.accounts.multi_delegate_authority.to_account_info(), 
        ctx.accounts.token_program.to_account_info())?;

    Ok(())
}
