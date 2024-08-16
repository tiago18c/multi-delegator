use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use crate::state::Delegation;

#[derive(Accounts)]
pub struct TransferRecurring<'info> {
    /// CHECK: will be checked in validate_recurring_transfer
    pub delegate: AccountInfo<'info>,

    #[account(mut,
        constraint = delegation.delegate == delegate.key(),
        seeds = [b"delegation", source.key().as_ref(), delegate.key().as_ref(),  &[Into::<u8>::into(&delegation.kind)] ],
        bump = delegation.bump
    )]
    pub delegation: Account<'info, Delegation>,

    /// CHECK: PDA, will be checked in CPI
    #[account(seeds = [b"multi_delegate", source.key().as_ref()], bump)]
    pub multi_delegate_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub source: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<TransferRecurring>) -> Result<()> {

    let (new_kind, new_state, amount) = ctx.accounts.delegation.validate_recurring_transfer(&ctx.accounts.destination.key())?;

    let seeds = [
        b"multi_delegate", 
        ctx.accounts.source.to_account_info().key.as_ref(), 
        &[ctx.bumps.multi_delegate_authority]];
    let signer_seeds = &[&seeds[..]];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx
                    .accounts
                    .source
                    .to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.multi_delegate_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    ctx.accounts.delegation.update_state(new_state);
    ctx.accounts.delegation.update_kind(new_kind);


    Ok(())
}
