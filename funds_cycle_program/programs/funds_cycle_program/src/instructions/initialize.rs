use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"config", admin.key().as_ref()],
        bump,
        space = 8 + ConfigAccount::INIT_SPACE
    )]
    pub config: Account<'info, ConfigAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [b"vault", config.key().as_ref()],
        bump,
        space = 8 + VaultAccount::INIT_SPACE
    )]
    pub vault: Account<'info, VaultAccount>,

    pub system_program: Program<'info, System>,
}
impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self,
        collateral_amount: u64,
        monthly_payout: u64,
        payment_interval_days: u16,
        max_beneficiaries: u8,
        withdraw_percent: u8,
        bumps: &InitializeBumps
    ) -> Result<()> {
        // Config setup
        self.config.set_inner(ConfigAccount {
            admin: self.admin.key(),
            collateral_amount,
            monthly_payout,
            payment_interval_days,
            withdraw_percent,
            max_beneficiaries,
            current_index: 0,
            claimable: false,
            claims_completed: 0,
            bump: bumps.config,
        });
        // Vault setup
        self.vault.set_inner(VaultAccount {
            config: self.config.key(),
            bump: bumps.vault,
        });

      
        Ok(())
    }
}
