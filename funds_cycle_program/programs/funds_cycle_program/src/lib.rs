#![allow(deprecated, unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod state;
pub mod error;
pub mod instructions;

pub use instructions::*;

declare_id!("BAmKovDnmFfuvXASrEoRa115N3F4QEBCkjUQtRAvkpAj");

#[program]
pub mod funds_cycle_program {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        collateral_amount: u64,
        monthly_payout: u64,
        payment_interval_days: u16,
        max_beneficiaries: u8,
        withdraw_percent: u8
    ) -> Result<()> {
        ctx.accounts.initialize(
            monthly_payout,
            collateral_amount,
            payment_interval_days,
            max_beneficiaries,
            withdraw_percent,
            &ctx.bumps
        )?;
        Ok(())
    }

    pub fn add_beneficiary(ctx: Context<AddBeneficiary>) -> Result<()> {
        ctx.accounts.add_beneficiary(&ctx.bumps)
    }

    pub fn deposit_collateral(ctx: Context<Deposit>) -> Result<()> {
        ctx.accounts.deposit_collateral()
    }

    pub fn deposit_monthly(ctx: Context<Deposit>) -> Result<()> {
        ctx.accounts.deposit_monthly()
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        ctx.accounts.withdraw()
    }

    pub fn punish(ctx: Context<Punish>) -> Result<()> {
        ctx.accounts.punish()
    }

    pub fn enable_claiming(ctx: Context<ClaimCollateral>) -> Result<()> {
        ctx.accounts.enable_claiming()?;
        Ok(())
    }
     pub fn claim_collateral(ctx: Context<ClaimCollateral>) -> Result<()> {
        ctx.accounts.claim_collateral()

    }

    pub fn exit(ctx: Context<Exit>) -> Result<()> {
        ctx.accounts.exit()
    }
}
