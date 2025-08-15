// instructions/add_beneficiary.rs
use anchor_lang::prelude::*;
use crate::{ error::FundCycleError, state::* };

#[derive(Accounts)]
pub struct AddBeneficiary<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin,
        constraint = config.current_index < config.max_beneficiaries @ FundCycleError::MaxBeneficiariesReached
    )]
    pub config: Account<'info, ConfigAccount>,

    /// This is the wallet we are adding as beneficiary
    /// Can be the same as `admin` or different — usually a different user.
    pub wallet: SystemAccount<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"beneficiary", config.key().as_ref(), wallet.key().as_ref()],
        bump,
        space = 8 + BeneficiaryAccount::INIT_SPACE
    )]
    pub beneficiary: Account<'info, BeneficiaryAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddBeneficiary<'info> {
    pub fn add_beneficiary(&mut self, bumps: &AddBeneficiaryBumps) -> Result<()> {
        let index = self.config.current_index; // Current slot in sequence

        // Save beneficiary info
        self.beneficiary.set_inner(BeneficiaryAccount {
            config: self.config.key(),
            wallet: self.wallet.key(),
            index,
            collateral_paid: false,
            monthly_paid: false,
            bump: bumps.beneficiary,
            last_payment_ts: 0, 
            active: true,
            collateral_claimed:false
        });

        // Increase count in config
        self.config.current_index += 1;

        Ok(())
    }
}
