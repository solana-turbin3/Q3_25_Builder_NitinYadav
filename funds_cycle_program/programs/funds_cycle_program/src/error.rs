use anchor_lang::prelude::*;

#[error_code]
pub enum FundCycleError {
    // ========= beneficiary error =====
    #[msg("Maximum number of beneficiaries reached")]
    MaxBeneficiariesReached,

    // ========= deposit error =====
    #[msg("Collateral already paid")]
    CollateralAlreadyPaid,
    #[msg("Collateral not paid")]
    CollateralNotPaid,
    #[msg("Monthly contribution already paid")]
    AlreadyPaidMonthly,

    // ========= withdraw error =====
    #[msg("It's not your turn to withdraw")]
    NotYourTurn,
    #[msg("Monthly contribution not paid")]
    MonthlyNotPaid,
    #[msg("Beneficiary is inactive")]
    InactiveBeneficiary,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("No funds available for withdrawal")]
    NoFundsAvailable,
    #[msg("Insufficient balance in vault")]
    InsufficientVaultBalance,

    // ========= punish.rs error =====
    #[msg("Payment is still on time, cannot punish")]
    PaymentStillOnTime,

    // ========= claim_collateral error =====
    #[msg("Invalid beneficiary wallet")]
    InvalidBeneficiaryWallet,
    #[msg("Claiming is already enabled")]
    AlreadyClaimable,
    #[msg("Round robin cycle not complete")]
    CycleNotComplete,
    #[msg("Missing beneficiary account")]
    MissingBeneficiaryAccount,
    #[msg("Collateral already claimed")]
    AlreadyClaimed,
    #[msg("Insufficient funds in vault")]
    InsufficientVaultFunds,

    // ========= exit error =====
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Claiming is not enabled")]
    ClaimingNotEnabled,
    #[msg("Not all beneficiaries have claimed")]
    NotAllClaimed,
    #[msg("Invalid config account")]
    InvalidConfig,
}