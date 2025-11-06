import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import * as fs from "fs";
import { Program } from "@coral-xyz/anchor";
import { EphemeralVault } from "../target/types/ephemeral_vault"

describe("ephemeral_vault (devnet)", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // ✅ Load IDL manually
  const idl = JSON.parse(
    fs.readFileSync("./target/idl/ephemeral_vault.json", "utf8")
  );

  // ✅ Create program instance
  const program = new anchor.Program(idl as anchor.Idl, provider) as Program<EphemeralVault>;


  const user = anchor.web3.Keypair.generate();
  const delegate = anchor.web3.Keypair.generate();
  const cleaner = anchor.web3.Keypair.generate();

  const approvedAmount = new anchor.BN(1_000_000_000);
  let vaultPda: anchor.web3.PublicKey;
  let bump: number;

  [vaultPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  it("Creates Ephemeral Vault", async () => {
    await program.methods
      .createEphemeralVault(approvedAmount)
      .accountsPartial({
        user: user.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);

    assert.strictEqual(vault.userWallet.toBase58(), user.publicKey.toBase58());
    assert.strictEqual(vault.isActive, true);
    assert.strictEqual(
      vault.approvedAmount.toNumber(),
      approvedAmount.toNumber()
    );
  });

  it("Approves Delegate", async () => {
    await program.methods
      .approveDelegate(delegate.publicKey)
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
      })
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);
    assert.ok(vault.delegateWallet);
    assert.strictEqual(vault.delegateWallet.toBase58(), delegate.publicKey.toBase58());
  });

  it("Auto Deposits SOL for Trade", async () => {
    const tradeFeeEstimate = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);

    await program.methods
      .autoDepositForTrade(tradeFeeEstimate)
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);
    assert.ok(vault.totalDeposited.toNumber() > 0);
    assert.ok(vault.lastActivity > 0);
  });

  it("Delegate Executes Trade", async () => {
    const tradeFee = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
    const tradeAmount = new anchor.BN(1_000_000);

    await program.methods
      .executeTrade(tradeFee, tradeAmount)
      .accounts({
        delegate: delegate.publicKey,
        vault: vaultPda,
      })
      .signers([delegate])
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);
    assert.strictEqual(vault.usedAmount.toNumber(), tradeAmount.toNumber());
  });

  it("User Revokes Access", async () => {
    await program.methods
      .revokeAccess()
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
      })
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);
    assert.strictEqual(vault.isActive, false);
    assert.strictEqual(vault.delegateWallet, null);
  });

    it("Reactivate Delegate", async () => {
    await program.methods
      .reactivateVault()
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
      })
      .rpc();
  })

   it("User Revokes Access", async () => {
    await program.methods
      .revokeAccess()
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
      })
      .rpc();

    const vault = await program.account.ephemeralVault.fetch(vaultPda);
    assert.strictEqual(vault.isActive, false);
    assert.strictEqual(vault.delegateWallet, null);
  });

  // it("Cleanup Vault after expiry", async () => {
  //   // simulate vault expiry delay
  //   await new Promise((resolve) => setTimeout(resolve, 2000));

  //   await program.methods
  //     .cleanupVault()
  //     .accounts({
  //       vault: vaultPda,
  //       userWallet: user.publicKey,
  //       cleaner: cleaner.publicKey,
  //     })
  //     .signers([cleaner])
  //     .rpc();

  //   const vault = await program.account.ephemeralVault.fetch(vaultPda);
  //   assert.strictEqual(vault.isActive, false);
  // });
});
