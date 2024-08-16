use anchor_lang::{prelude::*, solana_program::program_option::COption};
use anchor_spl::token_interface::{self, TokenAccount};

use crate::error::ErrorCode;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub enum DelegationKind {
    Simple(u64),
    Recurring(RecurringDelegation),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct RecurringDelegation {
    pub amount_per_period: u64,
    pub period: u64,
    pub start_date: u64,
    pub end_date: u64,
    pub last_claim: u64,
    pub destination: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Delegation {
    pub delegate: Pubkey,
    pub state: State,
    pub bump: u8,
    pub kind: DelegationKind,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace, PartialEq, Eq)]
pub enum State {
    Pending,
    Active,
    Inactive,
}

impl From<&DelegationKind> for u8 {
    fn from(kind: &DelegationKind) -> Self {
        match kind {
            DelegationKind::Simple(_) => 1u8,
            DelegationKind::Recurring(_) => 2u8,
        }
    }
}

impl Delegation {
    pub fn set_delegate_if_needed<'info>(token_account: &InterfaceAccount<'info, TokenAccount>, authority: AccountInfo<'info>, multi_delegate_authority: AccountInfo<'info> , token_program: AccountInfo<'info>) -> Result<()> {

        match token_account.delegate {
            COption::Some(delegate) if delegate == multi_delegate_authority.key() => (),
            _ => {
                token_interface::approve(
                CpiContext::new(
                    token_program,
                    token_interface::Approve {
                        to: token_account.to_account_info(),
                        delegate: multi_delegate_authority,
                        authority: authority,
                    },
                ),
                u64::MAX)?;
            }
        }

        Ok(())
    }

    pub fn validate_simple_transfer(&self, amount: u64) -> Result<(DelegationKind, State, u64)> {
        require!(self.state == State::Active, ErrorCode::InvalidState);

        match &self.kind {
            DelegationKind::Simple(max) => {
                require!(*max >= amount, ErrorCode::InsufficientAmount);

                let new_state = if *max == 0 {
                    State::Inactive
                } else {
                    State::Active
                };

                Ok((DelegationKind::Simple(*max - amount), new_state, amount))
            }
            _ => Err(ErrorCode::InvalidKind.into()),
        }
    }

    pub fn validate_recurring_transfer(&self, destination: &Pubkey) -> Result<(DelegationKind, State, u64)> {
        require!(self.state == State::Active, ErrorCode::InvalidState);

        match &self.kind {
            DelegationKind::Recurring(recurring_delegation) => {
                require!(recurring_delegation.destination == *destination, ErrorCode::InvalidDestination);
                let now = Clock::get()?.unix_timestamp as u64;
                let cutoff = now.min(recurring_delegation.end_date);

                let amount: u64 = ((now - recurring_delegation.last_claim) as u128)
                    .checked_mul(recurring_delegation.amount_per_period as u128)
                    .and_then(|x| x.checked_div(recurring_delegation.period as u128))
                    .and_then(|x| u64::try_from(x).ok()).unwrap();

                let mut result = recurring_delegation.clone();

                result.last_claim = cutoff;

                let new_state = if recurring_delegation.last_claim == recurring_delegation.end_date {
                    State::Inactive
                } else {
                    State::Active
                };

                Ok((DelegationKind::Recurring(result), new_state, amount))
            }
            _ => Err(ErrorCode::InvalidKind.into()),
        }
    }

    pub fn update_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub fn update_kind(&mut self, new_kind: DelegationKind) {
        self.kind = new_kind;
    }

    pub fn init_delegation(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp as u64;


        match &mut self.kind {
            DelegationKind::Recurring(recurring_delegation) => {
                require!(recurring_delegation.start_date < recurring_delegation.end_date, ErrorCode::InvalidDuration);

                if recurring_delegation.start_date <= now { // if its in the past, set last claim to now
                    recurring_delegation.last_claim = now;
                } else { // else set it to start date
                    recurring_delegation.last_claim = recurring_delegation.start_date;
                }
                
                Ok(())
            },
            _ => Ok(()),
        }
    }
}