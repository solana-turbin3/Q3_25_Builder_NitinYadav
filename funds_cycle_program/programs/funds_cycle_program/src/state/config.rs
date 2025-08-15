use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ConfigAccount {
    pub admin: Pubkey,
    pub collateral_amount: u64,
    pub monthly_payout: u64,
    pub payment_interval_days: u16, // e.g., 30 days
    pub withdraw_percent: u8,
    pub max_beneficiaries: u8,
    pub current_index: u8,
    pub bump: u8,
    pub claimable: bool,
    pub claims_completed: u8,
}
