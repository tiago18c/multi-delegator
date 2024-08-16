use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::{error::ErrorCode, state::Delegation, State};

#[derive(Accounts)]
pub struct RevokeDelegate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
        seeds = [b"delegation", token_account.key().as_ref(), delegation.delegate.as_ref(),  &[Into::<u8>::into(&delegation.kind)] ],
        bump = delegation.bump
    )]
    pub delegation: Account<'info, Delegation>,

    pub token_account: InterfaceAccount<'info, TokenAccount>,
}

pub fn handler(ctx: Context<RevokeDelegate>) -> Result<()> {
    require!(ctx.accounts.authority.key() == ctx.accounts.delegation.delegate
        || ctx.accounts.authority.key() == ctx.accounts.token_account.owner, ErrorCode::InvalidAuthority);


    let delegation = &mut ctx.accounts.delegation;
    delegation.state = State::Inactive;

    msg!("Delegation revoked for delegate: {:?}", delegation.delegate);
    Ok(())
}