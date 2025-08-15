use anchor_lang::prelude::*;
use anchor_lang::system_program::{ transfer, Transfer };
use crate::state::*;
use crate::error::FundCycleError;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The wallet (caller) who must be the beneficiary for this turn
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// Config PDA (seeds: ["config", admin_pubkey])
    #[account(
        mut,
        seeds = [b"config", config.admin.as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, ConfigAccount>,

    /// Beneficiary PDA for the caller
    /// seeds = ["beneficiary", config.key(), wallet.key()]
    #[account(
        mut,
        seeds = [b"beneficiary", config.key().as_ref(), wallet.key().as_ref()],
        bump = beneficiary.bump,
        constraint = beneficiary.wallet == wallet.key() @ FundCycleError::Unauthorized,
        constraint = beneficiary.config == config.key() @ FundCycleError::InvalidConfig,
        constraint = beneficiary.index == config.current_index @ FundCycleError::NotYourTurn,
        constraint = beneficiary.monthly_paid == true @ FundCycleError::MonthlyNotPaid
    )]
    pub beneficiary: Account<'info, BeneficiaryAccount>,

    /// Vault PDA holding lamports (seeds: ["vault", config.key()])
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultAccount>,

    /// System program for SOL transfer CPI
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        // basic sanity: must have at least 1 beneficiary configured

        require!(self.config.max_beneficiaries > 0, FundCycleError::InvalidConfig);
        require!(self.beneficiary.active == true, FundCycleError::InactiveBeneficiary);

        let clock = Clock::get()?;
        let next_withdraw_ts =
            self.beneficiary.last_payment_ts + (self.config.payment_interval_days as i64) * 86400;
        require!(clock.unix_timestamp >= next_withdraw_ts, FundCycleError::PaymentStillOnTime);

        // 1) Calculate monthly pool: monthly_payout * max_beneficiaries
        let monthly_pool = self.config.monthly_payout
            .checked_mul(self.config.max_beneficiaries as u64)
            .ok_or(FundCycleError::MathOverflow)?;

        // 2) Payout = monthly_pool * withdraw_percent / 100
        let payout_amount = monthly_pool
            .checked_mul(self.config.withdraw_percent as u64)
            .ok_or(FundCycleError::MathOverflow)?
            .checked_div(100)
            .ok_or(FundCycleError::MathOverflow)?;

        require!(payout_amount > 0, FundCycleError::NoFundsAvailable);

        // 3) Ensure vault has enough lamports (protects against under-deposit)
        let vault_lamports = self.vault.to_account_info().lamports();
        require!(vault_lamports >= payout_amount, FundCycleError::InsufficientVaultBalance);

        // 4) Transfer SOL from vault PDA -> caller wallet using signed CPI
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.wallet.to_account_info(),
        };

        // seeds used to derive vault PDA: [b"vault", config.key().as_ref(), &[vault.bump]]
        let seeds = &[
            b"vault".as_ref(),
            self.config.to_account_info().key.as_ref(),
            &[self.vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, payout_amount)?;

        // 5) Reset monthly flag for this beneficiary (they'll need to pay next month again)
        self.beneficiary.monthly_paid = false;

        // 6) Advance round-robin index (wrap around)
        self.config.current_index = (self.config.current_index + 1) % self.config.max_beneficiaries;

        Ok(())
    }
}
