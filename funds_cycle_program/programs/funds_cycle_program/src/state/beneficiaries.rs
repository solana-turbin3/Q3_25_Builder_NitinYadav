use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BeneficiaryAccount {
    pub config: Pubkey,          
    pub wallet: Pubkey,          
    pub bump: u8,                
    pub index: u8,                
    pub collateral_paid: bool,   
    pub monthly_paid: bool,      
    pub last_payment_ts: i64,    
    pub active: bool,            
    pub collateral_claimed: bool,
}
