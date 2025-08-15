# FundCycle Protocol

**FundCycle Protocol** is a decentralized group savings and payout system built on Solana using Anchor.  
It enables an admin to create fixed-term fund cycles with predefined participants, collateral requirements, and payout rules.

## Key Features
- **Admin Initialization** – Set up fund cycles with fixed participants and terms.
- **Collateral Requirement** – Participants deposit collateral upfront to join.
- **Monthly Contributions** – Members must pay on time to stay active.
- **Round Robin Payouts** – 80% of the vault is paid monthly to one participant in sequence.
- **Reserve Fund** – 20% of vault retained each month for yield generation & risk reduction.
- **Penalty System** – Late/missed payments mark members inactive & forfeit collateral.
- **Conditional Withdrawals** – Payouts & collateral released only after obligations are met.
- **Cycle Exit Rules** – Members can exit only after the full cycle ends and accounts are closed.
- **Platform Fee** – 1.5% fee on deposits & payouts, stored in a protocol fee vault.
- **Admin Fee Withdrawal** – Admin can withdraw collected fees for protocol maintenance.

---

## State Accounts

### `ConfigAccount`
Stores global fund cycle configuration:
- **admin** – Pubkey of protocol admin  
- **collateral_amount** – Required collateral per participant  
- **monthly_payout** – Monthly contribution amount  
- **payment_interval_days** – Payment interval (e.g., 30 days)  
- **withdraw_percent** – Payout percentage to winner each month  
- **max_beneficiaries** – Max participant slots  
- **current_index** – Tracks payout order  
- **claimable** – Whether payouts can be claimed  
- **claims_completed** – Completed payout count  
- **bump** – PDA bump

### `VaultAccount`
Holds the main vault for the cycle:
- **config** – Linked ConfigAccount  
- **bump** – PDA bump

### `BeneficiaryAccount`
Tracks participant data:
- **config** – Linked ConfigAccount  
- **wallet** – Participant wallet address  
- **index** – Position in payout order  
- **collateral_paid** – If collateral deposited  
- **monthly_paid** – If monthly payment made  
- **last_payment_ts** – Timestamp of last payment  
- **active** – Participation status  
- **collateral_claimed** – Whether collateral withdrawn  
- **bump** – PDA bump

---

## Overview of Entire Architecture

```mermaid
flowchart TB
    %% Admin Setup Phase
    Admin([👤 Admin]) -->|Initialize System| Setup{Setup Phase}
    Setup -->|Create Config PDA| Config[📋 Config Account<br/>• admin_pubkey<br/>• max_beneficiaries: 5<br/>• collateral_amount<br/>• monthly_amount<br/>• withdrawal_percent: 80%<br/>• fee_percent: 1.5%<br/>• current_index: 0]
    Setup -->|Create Main Vault PDA| Vault[🏦 Vault Account<br/>• Collateral Pool<br/>• Monthly Deposits<br/>• Reserve Fund 20%]
    Setup -->|Create Fee Vault PDA| FeeVault[💼 Fee Vault PDA<br/>• Protocol Platform Revenue Account]
    Setup -->|Generate Beneficiary Slots| Ben1[👥 Beneficiary 1<br/>• wallet_pubkey<br/>• is_active: false<br/>• collateral_paid: false<br/>• last_payment_slot<br/>• has_withdrawn: false]
    Setup --> Ben2[👥 Beneficiary 2]
    Setup --> Ben3[👥 Beneficiary 3]
    Setup --> Ben4[👥 Beneficiary 4]
    Setup --> Ben5[👥 Beneficiary 5]

    %% Participation Phase
    subgraph Participation["🚀 Participation Phase"]
        User([👤 Participant]) -->|Join| JoinCheck{Available Slot?}
        JoinCheck -->|Yes| PayCollateral[💰 Pay Collateral]
        JoinCheck -->|No| Rejected[❌ Rejected - Full]
        PayCollateral -->|1.5% Platform Fee| FeeVault
        PayCollateral -->|98.5% to Main Vault| Vault
        PayCollateral --> ActivateSlot[✅ Activate Beneficiary Slot]
    end

    %% Monthly Cycle
    subgraph MonthlyCycle["🗓️ Monthly Operations"]
        MonthStart([📅 Month Begins]) --> PaymentWindow{Payment Window<br/>Days 1-25}
        PaymentWindow -->|Pay Monthly Amount| MonthlyPayment[💵 Monthly Deposit]
        PaymentWindow -->|Miss Deadline| Punishment[⚠️ Punishment<br/>• Mark Inactive<br/>• Retain Collateral<br/>• Remove from Cycle]
        
        MonthlyPayment -->|1.5% Platform Fee| FeeVault
        MonthlyPayment -->|98.5% to Main Vault| Vault
        MonthlyPayment --> TriggerPayout{All Active Paid?}
        
        TriggerPayout -->|Yes| RoundRobin[🎯 Round Robin Payout<br/>80% of Pool to Winner]
        TriggerPayout -->|No| WaitForPayments[⏳ Wait for Payments]
        
        RoundRobin -->|1.5% Platform Fee on Payout| FeeVault
        RoundRobin -->|Remaining to Winner| CurrentWinner{Current Index}
        
        CurrentWinner -->|Index 0| Winner1[🏆 Beneficiary 1 Wins]
        CurrentWinner -->|Index 1| Winner2[🏆 Beneficiary 2 Wins]
        CurrentWinner -->|Index 2| Winner3[🏆 Beneficiary 3 Wins]
        CurrentWinner -->|Index 3| Winner4[🏆 Beneficiary 4 Wins]
        CurrentWinner -->|Index 4| Winner5[🏆 Beneficiary 5 Wins]
        
        Winner1 --> NextMonth[➡️ Next Month<br/>Index++]
        Winner2 --> NextMonth
        Winner3 --> NextMonth
        Winner4 --> NextMonth
        Winner5 --> CycleComplete
        
        Punishment --> Vault
        NextMonth --> MonthStart
    end

    %% Cycle Completion
    subgraph Completion["🏁 Cycle Completion"]
        CycleComplete[✅ All 5 Rounds Complete] --> EmptyVault[🔄 Empty Vault Process]
        EmptyVault --> SplitCollateral[💰 Return Collateral<br/>Split Among Active Beneficiaries]
        EmptyVault --> AdminFees[💼 Protocol Keeps<br/>• All Platform Fees in Fee Vault<br/>• Any Remaining Balance<br/>• Forfeited Collaterals]
        SplitCollateral --> CloseBen[🗑️ Close Beneficiary Accounts]
        CloseBen --> CloseVault[🗑️ Close Vault Account]
        CloseVault --> CloseFeeVault[🗑️ Close Fee Vault PDA]
        CloseFeeVault --> CloseConfig[🗑️ Close Config Account]
        CloseConfig --> SystemEnd[🔚 System Terminated]
    end

    %% Key Features Highlight
    subgraph Features["✨ Key Features"]
        F1[💡 Round Robin Selection<br/>Fair monthly winner rotation]
        F2[🔒 Collateral Security<br/>Ensures participant commitment]
        F3[⚡ Automated Punishment<br/>Inactive members lose collateral]
        F4[💸 Protocol Revenue Model<br/>1.5% platform fee on all money moves]
        F5[🎲 Slot-based System<br/>Maximum 5 participants]
    end

    %% Styling
    classDef admin fill:#8b5cf6,stroke:#7c3aed,stroke-width:3px,color:#fff
    classDef account fill:#1f2937,stroke:#374151,stroke-width:2px,color:#fff
    classDef user fill:#059669,stroke:#047857,stroke-width:2px,color:#fff
    classDef process fill:#dc2626,stroke:#b91c1c,stroke-width:2px,color:#fff
    classDef winner fill:#f59e0b,stroke:#d97706,stroke-width:3px,color:#fff
    classDef feature fill:#0ea5e9,stroke:#0284c7,stroke-width:2px,color:#fff
    classDef decision fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#fff

    class Admin,AdminFees admin
    class Config,Vault,Ben1,Ben2,Ben3,Ben4,Ben5,FeeVault account
    class User,PayCollateral,MonthlyPayment user
    class Punishment,EmptyVault,SplitCollateral process
    class Winner1,Winner2,Winner3,Winner4,Winner5,RoundRobin winner
    class F1,F2,F3,F4,F5 feature
    class JoinCheck,PaymentWindow,TriggerPayout,CurrentWinner decision
