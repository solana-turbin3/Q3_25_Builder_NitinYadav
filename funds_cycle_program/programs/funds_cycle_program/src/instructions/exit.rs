// instructions/exit.rs
use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::FundCycleError;

#[derive(Accounts)]
pub struct Exit<'info> {
    /// Only admin can exit the program
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"config", config.admin.as_ref()],
        bump = config.bump,
        constraint = config.admin == admin.key() @ FundCycleError::Unauthorized,
        constraint = config.claimable @ FundCycleError::ClaimingNotEnabled,
        constraint = config.claims_completed >= config.max_beneficiaries @ FundCycleError::NotAllClaimed,
        close = admin  // Close config account and send rent to admin
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = vault.bump,
        constraint = vault.config == config.key() @ FundCycleError::InvalidConfig,
        close = admin  // Close vault account and send remaining funds + rent to admin
    )]
    pub vault: Account<'info, VaultAccount>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> Exit<'info> {
    pub fn exit(&mut self) -> Result<()> {
        // Double-check all claims are completed
        require!(
            self.config.claims_completed == self.config.max_beneficiaries,
            FundCycleError::NotAllClaimed
        );
        
        // Get remaining vault balance before closing
        let remaining_balance = self.vault.to_account_info().lamports();
        
        msg!(
            "PROGRAM EXIT: All {} beneficiaries claimed collateral successfully!",
            self.config.max_beneficiaries
        );
        
        msg!(
            "Returning remaining vault balance of {} lamports to admin: {}",
            remaining_balance,
            self.admin.key()
        );
        
        msg!(" Closing all program accounts and returning rent to admin");
        
        // Accounts will be automatically closed due to close constraints
        // Vault balance + rent will go to admin
        
        Ok(())
    }
}