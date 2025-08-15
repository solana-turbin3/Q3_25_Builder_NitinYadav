use anchor_lang::prelude::*;
use crate::state::{ConfigAccount, BeneficiaryAccount};
use crate::error::FundCycleError;

#[derive(Accounts)]
pub struct Punish<'info> {
    /// Only admin can punish
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"config", config.admin.as_ref()],
        bump = config.bump,
        constraint = config.admin == admin.key() @ FundCycleError::Unauthorized
    )]
    pub config: Account<'info, ConfigAccount>,

    #[account(
        mut,
        seeds = [b"beneficiary", config.key().as_ref(), beneficiary.wallet.as_ref()],
        bump = beneficiary.bump,
        constraint = beneficiary.config == config.key() @ FundCycleError::InvalidConfig
    )]
    pub beneficiary: Account<'info, BeneficiaryAccount>,
}

impl<'info> Punish<'info> {
    pub fn punish(&mut self) -> Result<()> {
        let clock = Clock::get()?;

        // Calculate next due date
        let payment_due_ts = self.beneficiary.last_payment_ts
            + (self.config.payment_interval_days as i64 * 86400); // days → seconds

        // Check if overdue
        require!(
            clock.unix_timestamp > payment_due_ts,
            FundCycleError::PaymentStillOnTime
        );

        // Mark inactive (punished)
        self.beneficiary.active = false;

        Ok(())
    }
}
