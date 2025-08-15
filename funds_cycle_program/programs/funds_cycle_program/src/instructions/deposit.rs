use anchor_lang::prelude::*;
use anchor_lang::system_program::{ transfer, Transfer };
use crate::state::{ ConfigAccount, VaultAccount, BeneficiaryAccount };
use crate::error::FundCycleError;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub wallet: Signer<'info>,

    #[account(seeds = [b"config", config.admin.as_ref()], bump = config.bump)]
    pub config: Account<'info, ConfigAccount>,

    #[account(
        mut,
        seeds = [b"beneficiary", config.key().as_ref(), wallet.key().as_ref()],
        bump = beneficiary.bump,
        constraint = beneficiary.wallet == wallet.key() @ FundCycleError::Unauthorized,
        constraint = beneficiary.config == config.key() @ FundCycleError::InvalidConfig
    )]
    pub beneficiary: Account<'info, BeneficiaryAccount>,

    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit_collateral(&mut self) -> Result<()> {
        require!(!self.beneficiary.collateral_paid, FundCycleError::CollateralAlreadyPaid);

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.wallet.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, self.config.collateral_amount)?;

        self.beneficiary.collateral_paid = true;
        Ok(())
    }

    pub fn deposit_monthly(&mut self) -> Result<()> {
        require!(self.beneficiary.collateral_paid, FundCycleError::CollateralNotPaid);
        require!(self.beneficiary.active, FundCycleError::InactiveBeneficiary);
        require!(!self.beneficiary.monthly_paid, FundCycleError::AlreadyPaidMonthly);

        // Process payment
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.wallet.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, self.config.monthly_payout)?;

        let clock = Clock::get()?;
        self.beneficiary.monthly_paid = true;
        self.beneficiary.last_payment_ts = clock.unix_timestamp;
        Ok(())
    }
}
