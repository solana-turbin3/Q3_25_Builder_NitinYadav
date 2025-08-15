// instructions/claim_collateral.rs
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::system_program::transfer;
use crate::state::{ConfigAccount, BeneficiaryAccount, VaultAccount};
use crate::error::FundCycleError;

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {
    /// Admin or Beneficiary - depending on which function is called
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"config", config.admin.as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = vault.bump,
        constraint = vault.config == config.key() @ FundCycleError::InvalidConfig
    )]
    pub vault: Account<'info, VaultAccount>,
    
    /// Optional - only needed for claim_collateral function
    #[account(
        mut,
        seeds = [b"beneficiary", config.key().as_ref(), signer.key().as_ref()],
        bump,
        constraint = beneficiary.config == config.key() @ FundCycleError::InvalidConfig,
        constraint = beneficiary.wallet == signer.key() @ FundCycleError::InvalidBeneficiaryWallet
    )]
    pub beneficiary: Option<Account<'info, BeneficiaryAccount>>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimCollateral<'info> {
    pub fn enable_claiming(&mut self) -> Result<()> {
        // Verify admin authorization
        require!(
            self.config.admin == self.signer.key(),
            FundCycleError::Unauthorized
        );
        
        // Check if claiming is already enabled
        require!(
            !self.config.claimable,
            FundCycleError::AlreadyClaimable
        );
        
        // Check if round robin cycle is complete
        require!(
            self.config.current_index >= self.config.max_beneficiaries,
            FundCycleError::CycleNotComplete
        );
        
        // Enable claiming for all beneficiaries
        self.config.claimable = true;
        
        msg!(
            "Round robin cycle complete! Claiming enabled for all {} beneficiaries.",
            self.config.max_beneficiaries
        );
        
        Ok(())
    }
    
    pub fn claim_collateral(&mut self) -> Result<()> {
        // Ensure claiming is enabled
        require!(
            self.config.claimable,
            FundCycleError::ClaimingNotEnabled
        );
        
        // Ensure beneficiary account is provided
        let beneficiary = self.beneficiary.as_mut()
            .ok_or(FundCycleError::MissingBeneficiaryAccount)?;
        
        // Validate beneficiary constraints
        require!(
            beneficiary.collateral_paid,
            FundCycleError::CollateralNotPaid
        );
        require!(
            beneficiary.active,
            FundCycleError::InactiveBeneficiary
        );
        require!(
            !beneficiary.collateral_claimed,
            FundCycleError::AlreadyClaimed
        );
        
        // Validate vault has enough funds
        let vault_balance = self.vault.to_account_info().lamports();
        require!(
            vault_balance >= self.config.collateral_amount,
            FundCycleError::InsufficientVaultFunds
        );
        
        // Transfer collateral from vault to beneficiary
        let transfer_cpi = system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };
        
        let config_key = self.config.key();
        let seeds = &[
            b"vault",
            config_key.as_ref(),
            &[self.vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            transfer_cpi,
            signer_seeds,
        );
        
        transfer(cpi_ctx, self.config.collateral_amount)?;
        
        // Mark as claimed
        beneficiary.collateral_claimed = true;
        
        // Increment claims completed counter with overflow protection
        self.config.claims_completed = self.config.claims_completed
            .checked_add(1)
            .ok_or(FundCycleError::MathOverflow)?;
        
        msg!(
            "Collateral of {} lamports returned to beneficiary: {} | Progress: {}/{}",
            self.config.collateral_amount,
            self.signer.key(),
            self.config.claims_completed,
            self.config.max_beneficiaries
        );
        
        // Check if all beneficiaries have claimed
        if self.config.claims_completed >= self.config.max_beneficiaries {
            msg!("🎉 All beneficiaries have successfully claimed their collateral!");
        }
        
        Ok(())
    }
    
    // ==================== 3. HELPER: CHECK CLAIMING STATUS ====================
    pub fn get_status(&self) -> (bool, u8, u8, u8, u64) {
        let vault_balance = self.vault.to_account_info().lamports();
        
        (
            self.config.claimable,                   // Can beneficiaries claim?
            self.config.current_index,               // Round robin position
            self.config.claims_completed,            // How many have claimed
            self.config.max_beneficiaries,           // Total beneficiaries
            vault_balance,                           // Remaining vault balance
        )
    }
    
    pub fn is_cycle_complete(&self) -> bool {
        self.config.current_index >= self.config.max_beneficiaries
    }
    
    pub fn is_all_claimed(&self) -> bool {
        self.config.claims_completed >= self.config.max_beneficiaries
    }
    
    pub fn get_claim_progress(&self) -> f32 {
        if self.config.max_beneficiaries == 0 {
            return 0.0;
        }
        (self.config.claims_completed as f32 / self.config.max_beneficiaries as f32) * 100.0
    }
}