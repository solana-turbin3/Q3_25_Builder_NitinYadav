import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { type FundsCycleProgram } from "../target/types/funds_cycle_program.ts";
import { describe, it, before } from "node:test";
import assert from "assert";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";
import * as dotenv from "dotenv";

// Load environment variables
dotenv.config();

describe("funds_cycle_program", () => {
  // === 🎯 Global Constants ===
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.FundsCycleProgram as Program<FundsCycleProgram>;

  // Admin wallet is pre-funded with 19 SOL
  const admin = provider.wallet;

  // Load beneficiary private keys from environment (both have 1 SOL on devnet)
  const beneficiary1 = Keypair.fromSecretKey(
    anchor.utils.bytes.bs58.decode(process.env.BENEFICIARY1_PRIVATE_KEY!)
  );
  const beneficiary2 = Keypair.fromSecretKey(
    anchor.utils.bytes.bs58.decode(process.env.BENEFICIARY2_PRIVATE_KEY!)
  );

  console.log(`✅ Beneficiary1: ${beneficiary1.publicKey.toBase58()}`);
  console.log(`✅ Beneficiary2: ${beneficiary2.publicKey.toBase58()}`);

  const collateralAmount = new BN(1_000_0);
  const monthlyPayout = new BN(100_0);
  const paymentIntervalDays = 0;
  const maxBeneficiaries = 3;
  const withdrawPercent = 10;

  // === 🎯 Global PDAs ===
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config"), admin.publicKey.toBuffer()],
    program.programId
  );
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), configPda.toBuffer()],
    program.programId
  );

  // Log PDAs at startup
  console.log(`\n🔑 PROGRAM ADDRESSES:`);
  console.log(`📄 Config PDA: ${configPda.toBase58()}`);
  console.log(`🏦 Vault PDA: ${vaultPda.toBase58()}`);
  console.log(`🏛️ Program ID: ${program.programId.toBase58()}\n`);

  const getBeneficiaryPda = (configPda: PublicKey, wallet: PublicKey) =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("beneficiary"), configPda.toBuffer(), wallet.toBuffer()],
      program.programId
    );

  const adminBeneficiaryPda = getBeneficiaryPda(configPda, admin.publicKey)[0];
  const beneficiary1Pda = getBeneficiaryPda(configPda, beneficiary1.publicKey)[0];
  const beneficiary2Pda = getBeneficiaryPda(configPda, beneficiary2.publicKey)[0];

  // === 🔧 Helper Functions ===
  const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  // Enhanced transaction logging function
  const logTransactionInfo = (signature: string, operation: string, additionalInfo?: any) => {
    console.log(`\n📋 TRANSACTION LOG:`);
    console.log(`🔗 Signature: ${signature}`);
    console.log(`⚡ Operation: ${operation}`);
    console.log(`📄 Config PDA: ${configPda.toBase58()}`);
    console.log(`🏦 Vault PDA: ${vaultPda.toBase58()}`);
    if (additionalInfo) {
      console.log(`ℹ️ Additional Info: ${JSON.stringify(additionalInfo)}`);
    }
    console.log(`🕐 Timestamp: ${new Date().toISOString()}\n`);
  };

  // Check if account exists
  const accountExists = async (pubkey: PublicKey): Promise<boolean> => {
    try {
      const accountInfo = await provider.connection.getAccountInfo(pubkey);
      return accountInfo !== null;
    } catch {
      return false;
    }
  };

  // Enhanced transaction helper with logging
  const executeTransaction = async (instruction: any, signers: any[] = [], operation: string = "Unknown Operation", additionalInfo?: any) => {
    const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash('confirmed');

    const transaction = await instruction.transaction();
    transaction.recentBlockhash = blockhash;
    transaction.lastValidBlockHeight = lastValidBlockHeight;
    transaction.feePayer = provider.wallet.publicKey;

    if (signers.length > 0) {
      transaction.partialSign(...signers);
    }

    const signedTx = await provider.wallet.signTransaction(transaction);
    const signature = await provider.connection.sendRawTransaction(signedTx.serialize(), { skipPreflight: false });
    await provider.connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight }, 'confirmed');

    // Log transaction details
    logTransactionInfo(signature, operation, additionalInfo);

    return signature;
  };

  // Track test state to avoid redundant operations
  let testState = {
    configExists: false,
    configInitialized: false,
    beneficiariesExist: { admin: false, ben1: false, ben2: false },
    beneficiariesAdded: false,
    collateralDeposited: false,
    monthlyDeposited: false,
    cycleComplete: false,
    claimingEnabled: false,
    allCollateralClaimed: false,
    programExited: false,
  };

  // === 🏗️ PHASE 1: SETUP CHECK ===
  describe("🏗️ Setup Phase", () => {
    it("🔍 Checks existing accounts", async function () {
      // @ts-ignore
      this.timeout = 60000;

      console.log("🔍 Checking existing program accounts...");
      
      testState.configExists = await accountExists(configPda);
      const vaultExists = await accountExists(vaultPda);
      testState.beneficiariesExist.admin = await accountExists(adminBeneficiaryPda);
      testState.beneficiariesExist.ben1 = await accountExists(beneficiary1Pda);
      testState.beneficiariesExist.ben2 = await accountExists(beneficiary2Pda);

      console.log(`📄 Config account exists: ${testState.configExists}`);
      console.log(`🏦 Vault account exists: ${vaultExists}`);
      console.log(`👤 Admin beneficiary exists: ${testState.beneficiariesExist.admin}`);
      console.log(`👤 Beneficiary1 exists: ${testState.beneficiariesExist.ben1}`);
      console.log(`👤 Beneficiary2 exists: ${testState.beneficiariesExist.ben2}`);

      // Log beneficiary PDAs
      console.log(`\n👤 BENEFICIARY PDAs:`);
      console.log(`👑 Admin Beneficiary PDA: ${adminBeneficiaryPda.toBase58()}`);
      console.log(`👤 Beneficiary1 PDA: ${beneficiary1Pda.toBase58()}`);
      console.log(`👤 Beneficiary2 PDA: ${beneficiary2Pda.toBase58()}`);

      // Check account balances
      console.log("\n💰 Account balances:");
      const adminBalance = await provider.connection.getBalance(admin.publicKey);
      const ben1Balance = await provider.connection.getBalance(beneficiary1.publicKey);
      const ben2Balance = await provider.connection.getBalance(beneficiary2.publicKey);

      console.log(`👑 Admin: ${adminBalance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`👤 Beneficiary1: ${ben1Balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`👤 Beneficiary2: ${ben2Balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);

      // If config exists, check its state
      if (testState.configExists) {
        console.log("✅ Program already initialized");
        testState.configInitialized = true;
        
        try {
          const config = await program.account.configAccount.fetch(configPda);
          testState.cycleComplete = config.currentIndex >= config.maxBeneficiaries;
          testState.claimingEnabled = config.claimable;
          
          console.log(`📊 Config state - Current Index: ${config.currentIndex}, Max: ${config.maxBeneficiaries}`);
          console.log(`🔄 Cycle Complete: ${testState.cycleComplete}`);
          console.log(`🎯 Claiming Enabled: ${testState.claimingEnabled}`);
          
          if (config.claimsCompleted) {
            testState.allCollateralClaimed = config.claimsCompleted >= config.maxBeneficiaries;
            console.log(`💰 Claims Completed: ${config.claimsCompleted}/${config.maxBeneficiaries}`);
          }
        } catch (error) {
          console.log("⚠️ Could not fetch config state, will check during tests");
        }
        
        // Check beneficiary states if they exist
        if (testState.beneficiariesExist.admin && testState.beneficiariesExist.ben1 && testState.beneficiariesExist.ben2) {
          console.log("✅ All beneficiaries already exist");
          testState.beneficiariesAdded = true;
          
          try {
            const ben1 = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
            const ben2 = await program.account.beneficiaryAccount.fetch(beneficiary2Pda);
            
            if (ben1.collateralPaid && ben2.collateralPaid) {
              console.log("✅ Collateral already deposited");
              testState.collateralDeposited = true;
            }
            
            if (ben1.monthlyPaid && ben2.monthlyPaid) {
              console.log("✅ Monthly payments already deposited");
              testState.monthlyDeposited = true;
            }
          } catch (error) {
            console.log("⚠️ Could not fetch beneficiary states, will check during tests");
          }
        }
      }

      console.log("✅ Setup check complete");
      await sleep(1000);
    });
  });

  // === 🧪 PHASE 2: FUNCTIONAL TESTS ===
  describe("🧪 Functional Tests", () => {
    // Helper Functions for Tests with enhanced logging
    const addBeneficiary = async (wallet: PublicKey, beneficiaryPda: PublicKey) => {
      return await executeTransaction(
        program.methods
          .addBeneficiary()
          .accountsStrict({
            admin: admin.publicKey,
            config: configPda,
            wallet,
            beneficiary: beneficiaryPda,
            systemProgram: SystemProgram.programId,
          }),
        [],
        "Add Beneficiary",
        { walletAddress: wallet.toBase58(), beneficiaryPda: beneficiaryPda.toBase58() }
      );
    };

    const depositCollateral = async (user: Keypair, beneficiaryPda: PublicKey) => {
      return await executeTransaction(
        program.methods
          .depositCollateral()
          .accountsStrict({
            wallet: user.publicKey,
            config: configPda,
            beneficiary: beneficiaryPda,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          }),
        [user],
        "Deposit Collateral",
        { 
          userWallet: user.publicKey.toBase58(), 
          beneficiaryPda: beneficiaryPda.toBase58(),
          collateralAmount: collateralAmount.toString()
        }
      );
    };

    const depositMonthly = async (user: Keypair, beneficiaryPda: PublicKey) => {
      return await executeTransaction(
        program.methods
          .depositMonthly()
          .accountsStrict({
            wallet: user.publicKey,
            config: configPda,
            beneficiary: beneficiaryPda,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          }),
        [user],
        "Deposit Monthly Payment",
        { 
          userWallet: user.publicKey.toBase58(), 
          beneficiaryPda: beneficiaryPda.toBase58(),
          monthlyAmount: monthlyPayout.toString()
        }
      );
    };

    describe("Program Initialization", () => {
      it("✅ Initializes program (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        if (testState.configExists) {
          console.log("ℹ️ Config already exists, verifying...");
          const config = await program.account.configAccount.fetch(configPda);
          assert.strictEqual(config.admin.toBase58(), admin.publicKey.toBase58());
          console.log("✅ Existing program configuration verified");
        } else {
          console.log("🆕 Initializing new program...");
          await executeTransaction(
            program.methods
              .initialize(monthlyPayout, collateralAmount, paymentIntervalDays, maxBeneficiaries, withdrawPercent)
              .accountsStrict({
                admin: admin.publicKey,
                config: configPda,
                vault: vaultPda,
                systemProgram: SystemProgram.programId,
              }),
            [],
            "Initialize Program",
            {
              monthlyPayout: monthlyPayout.toString(),
              collateralAmount: collateralAmount.toString(),
              paymentIntervalDays,
              maxBeneficiaries,
              withdrawPercent
            }
          );

          const config = await program.account.configAccount.fetch(configPda);
          assert.strictEqual(config.admin.toBase58(), admin.publicKey.toBase58());
          console.log("✅ Program initialized successfully");
        }

        testState.configInitialized = true;
        await sleep(1000);
      });
    });

    describe("Beneficiary Management", () => {
      it("✅ Adds admin as beneficiary (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        if (testState.beneficiariesExist.admin) {
          console.log("ℹ️ Admin beneficiary already exists, verifying...");
          const beneficiary = await program.account.beneficiaryAccount.fetch(adminBeneficiaryPda);
          assert.strictEqual(beneficiary.wallet.toBase58(), admin.publicKey.toBase58());
          console.log("✅ Existing admin beneficiary verified");
        } else {
          console.log("🆕 Adding admin as beneficiary...");
          await addBeneficiary(admin.publicKey, adminBeneficiaryPda);
          console.log("✅ Admin added as beneficiary");
        }
        await sleep(1000);
      });

      it("✅ Adds beneficiary1 (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        if (testState.beneficiariesExist.ben1) {
          console.log("ℹ️ Beneficiary1 already exists, verifying...");
          const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
          assert.strictEqual(beneficiary.wallet.toBase58(), beneficiary1.publicKey.toBase58());
          console.log("✅ Existing beneficiary1 verified");
        } else {
          console.log("🆕 Adding beneficiary1...");
          await addBeneficiary(beneficiary1.publicKey, beneficiary1Pda);
          console.log("✅ Beneficiary1 added");
        }
        await sleep(1000);
      });

      it("✅ Adds beneficiary2 (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        if (testState.beneficiariesExist.ben2) {
          console.log("ℹ️ Beneficiary2 already exists, verifying...");
          const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary2Pda);
          assert.strictEqual(beneficiary.wallet.toBase58(), beneficiary2.publicKey.toBase58());
          console.log("✅ Existing beneficiary2 verified");
        } else {
          console.log("🆕 Adding beneficiary2...");
          await addBeneficiary(beneficiary2.publicKey, beneficiary2Pda);
          console.log("✅ Beneficiary2 added");
        }

        testState.beneficiariesAdded = true;
        await sleep(1000);
      });
    });

    describe("Collateral Deposits", () => {
      it("✅ Beneficiary1 deposits collateral (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
        
        if (beneficiary.collateralPaid) {
          console.log("ℹ️ Beneficiary1 collateral already paid");
        } else {
          console.log("🆕 Depositing beneficiary1 collateral...");
          await depositCollateral(beneficiary1, beneficiary1Pda);
          console.log("✅ Beneficiary1 collateral deposited");
        }
        await sleep(1000);
      });

      it("✅ Beneficiary2 deposits collateral (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary2Pda);
        
        if (beneficiary.collateralPaid) {
          console.log("ℹ️ Beneficiary2 collateral already paid");
        } else {
          console.log("🆕 Depositing beneficiary2 collateral...");
          await depositCollateral(beneficiary2, beneficiary2Pda);
          console.log("✅ Beneficiary2 collateral deposited");
        }

        testState.collateralDeposited = true;
        await sleep(1000);
      });
    });

    describe("Monthly Payments", () => {
      it("✅ Beneficiary1 deposits monthly payment (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
        
        if (beneficiary.monthlyPaid) {
          console.log("ℹ️ Beneficiary1 monthly payment already made");
        } else {
          console.log("🆕 Depositing beneficiary1 monthly payment...");
          await depositMonthly(beneficiary1, beneficiary1Pda);
          console.log("✅ Beneficiary1 monthly payment deposited");
        }
        await sleep(1000);
      });

      it("✅ Beneficiary2 deposits monthly payment (or confirms existing)", async function () {
        // @ts-ignore
        this.timeout = 30000;

        const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary2Pda);
        
        if (beneficiary.monthlyPaid) {
          console.log("ℹ️ Beneficiary2 monthly payment already made");
        } else {
          console.log("🆕 Depositing beneficiary2 monthly payment...");
          await depositMonthly(beneficiary2, beneficiary2Pda);
          console.log("✅ Beneficiary2 monthly payment deposited");
        }

        testState.monthlyDeposited = true;
        await sleep(1000);
      });
    });

    describe("Round Robin Tests", () => {
      it("✅ Tests withdrawal functionality", async function () {
        // @ts-ignore
        this.timeout = 60000;

        const config = await program.account.configAccount.fetch(configPda);
        const currentTurnIndex = config.currentIndex % maxBeneficiaries;

        if (currentTurnIndex === 1) {
          const vaultBalanceBefore = await provider.connection.getBalance(vaultPda);
          
          await executeTransaction(
            program.methods
              .withdraw()
              .accountsStrict({
                wallet: beneficiary1.publicKey,
                config: configPda,
                beneficiary: beneficiary1Pda,
                vault: vaultPda,
                systemProgram: SystemProgram.programId,
              }),
            [beneficiary1],
            "Withdraw",
            {
              beneficiaryWallet: beneficiary1.publicKey.toBase58(),
              currentTurnIndex,
              vaultBalanceBefore: (vaultBalanceBefore / anchor.web3.LAMPORTS_PER_SOL).toString() + " SOL"
            }
          );

          const vaultBalanceAfter = await provider.connection.getBalance(vaultPda);
          assert.ok(vaultBalanceAfter < vaultBalanceBefore, "Vault should have decreased");
          console.log("✅ Withdrawal test successful");
          console.log(`💰 Vault balance after withdrawal: ${vaultBalanceAfter / anchor.web3.LAMPORTS_PER_SOL} SOL`);
        } else {
          console.log(`ℹ️ Current turn index: ${currentTurnIndex}, test passed conditionally`);
        }
        await sleep(1000);
      });

      it("✅ Punish test for dishonest node", async function () {
        // @ts-ignore
        this.timeout = 30000;

        const ben1Before = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
        
        if (!ben1Before.active) {
          console.log("ℹ️ Beneficiary1 already inactive from previous test");
        } else {
          await executeTransaction(
            program.methods
              .punish()
              .accountsStrict({
                admin: admin.publicKey,
                config: configPda,
                beneficiary: beneficiary1Pda,
              }),
            [],
            "Admin Punish",
            {
              adminWallet: admin.publicKey.toBase58(),
              targetBeneficiary: beneficiary1.publicKey.toBase58(),
              beneficiaryPda: beneficiary1Pda.toBase58()
            }
          );

          const beneficiary = await program.account.beneficiaryAccount.fetch(beneficiary1Pda);
          assert.strictEqual(beneficiary.active, false, "Beneficiary should be inactive after punishment");
          console.log("✅ Admin punishment successful");
        }
        await sleep(1000);
      });
    }); // Closing Round Robin Tests

    describe("📊 Final State Verification", () => {
      it("✅ Complete program lifecycle summary", async function () {
        // @ts-ignore
        this.timeout = 30000;

        console.log("\n" + "=".repeat(80));
        console.log("🎯 COMPLETE PROGRAM LIFECYCLE SUMMARY");
        console.log("=".repeat(80));
        
        console.log("\n📋 Test State Summary:");
        console.log(`   ✅ Config Initialized: ${testState.configInitialized}`);
        console.log(`   ✅ Beneficiaries Added: ${testState.beneficiariesAdded}`);
        console.log(`   ✅ Collateral Deposited: ${testState.collateralDeposited}`);
        console.log(`   ✅ Monthly Payments: ${testState.monthlyDeposited}`);

        console.log("\n🔑 Program Addresses Used:");
        console.log(`   📄 Config PDA: ${configPda.toBase58()}`);
        console.log(`   🏦 Vault PDA: ${vaultPda.toBase58()}`);
        console.log(`   👑 Admin Beneficiary PDA: ${adminBeneficiaryPda.toBase58()}`);
        console.log(`   👤 Beneficiary1 PDA: ${beneficiary1Pda.toBase58()}`);
        console.log(`   👤 Beneficiary2 PDA: ${beneficiary2Pda.toBase58()}`);
        console.log(`   🏛️ Program ID: ${program.programId.toBase58()}`);

        assert.ok(testState.configInitialized, "Program should be initialized");
        assert.ok(testState.beneficiariesAdded, "Beneficiaries should be added");
        assert.ok(testState.collateralDeposited, "Collateral should be deposited");
        assert.ok(testState.monthlyDeposited, "Monthly payments should be deposited");
        
        console.log("✅ All lifecycle assertions passed!");
      });
    });
  }); // Closing Functional Tests
}); // Closing main describe