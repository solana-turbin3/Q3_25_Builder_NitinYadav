use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultAccount {
    pub config: Pubkey,
    pub bump: u8,
}


