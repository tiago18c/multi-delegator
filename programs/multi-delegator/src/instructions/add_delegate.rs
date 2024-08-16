use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use crate::{error::ErrorCode, state::{Delegation, DelegationKind, State}};


#[derive(Accounts)]
#[instruction(kind: DelegationKind)]
pub struct AddDelegate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub delegate: AccountInfo<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Delegation::INIT_SPACE,
        seeds = [b"delegation", token_account.key().as_ref(), delegate.key().as_ref(),  &[Into::<u8>::into(&kind)] ],
        bump
    )]
    pub delegation: Account<'info, Delegation>,

    /// CHECK: just the auth
    #[account(seeds = [b"multi_delegate", token_account.key().as_ref()], bump)]
    pub multi_delegate_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<AddDelegate>, kind: DelegationKind) -> Result<()> {
    require!(ctx.accounts.authority.key() == ctx.accounts.authority.key()
        || ctx.accounts.authority.key() == ctx.accounts.token_account.owner, ErrorCode::InvalidAuthority);

    let delegation = &mut ctx.accounts.delegation;
    delegation.delegate = ctx.accounts.delegate.key();
    delegation.kind = kind;
    delegation.init_delegation()?;
    delegation.bump = ctx.bumps.delegation;
    delegation.state = if ctx.accounts.authority.key() == ctx.accounts.token_account.owner.key() {
        Delegation::set_delegate_if_needed(
            &ctx.accounts.token_account, 
            ctx.accounts.authority.to_account_info(), 
            ctx.accounts.multi_delegate_authority.to_account_info(), 
            ctx.accounts.token_program.to_account_info())?;
        State::Active
    } else {
        State::Pending
    };



    Ok(())
}
