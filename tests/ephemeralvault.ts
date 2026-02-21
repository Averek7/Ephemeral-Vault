import * as anchor from "@coral-xyz/anchor";
import { AnchorError, BN, Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import * as fs from "fs";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { EphemeralVault } from "../target/types/ephemeral_vault";

const MIN_APPROVED_AMOUNT = new BN(1_000_000);
const MAX_APPROVED_AMOUNT = new BN(1_000_000_000_000);
const MIN_DEPOSIT_AMOUNT = new BN(1_000_000);
const MAX_DEPOSIT_AMOUNT = new BN(100_000_000_000);
const SESSION_DURATION_SECONDS = 3600;
const RENEWAL_WINDOW_SECONDS = 300;
const PROGRAM_VERSION = 1;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function airdrop(
  provider: anchor.AnchorProvider,
  pubkey: PublicKey,
  sol = 5,
) {
  const sig = await provider.connection.requestAirdrop(
    pubkey,
    sol * LAMPORTS_PER_SOL,
  );
  await provider.connection.confirmTransaction(sig, "confirmed");
}

function deriveVaultPda(
  programId: PublicKey,
  user: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user.toBuffer()],
    programId,
  );
}

function getErrorCode(err: unknown): string {
  if (err instanceof AnchorError) {
    return err.error.errorCode.code;
  }

  const msg = String((err as { message?: string })?.message ?? err);
  const match = msg.match(/Error Code: ([A-Za-z0-9_]+)/);
  if (match) return match[1];

  throw err;
}

async function expectError(promise: Promise<unknown>, code: string) {
  try {
    await promise;
    assert.fail(`Expected ${code}`);
  } catch (err) {
    assert.strictEqual(getErrorCode(err), code);
  }
}

type Fixture = {
  user: Keypair;
  delegate: Keypair;
  attacker: Keypair;
  cleaner: Keypair;
  vaultPda: PublicKey;
  bump: number;
};

describe("ephemeral_vault (TypeScript)", () => {
  const provider = process.env.ANCHOR_PROVIDER_URL
    ? anchor.AnchorProvider.env()
    : anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const idl = JSON.parse(
    fs.readFileSync("./target/idl/ephemeral_vault.json", "utf8"),
  );
  const program = new anchor.Program(
    idl as anchor.Idl,
    provider,
  ) as Program<EphemeralVault>;

  async function createFixture(
    approvedAmount = new BN(2 * LAMPORTS_PER_SOL),
  ): Promise<Fixture> {
    const user = Keypair.generate();
    const delegate = Keypair.generate();
    const attacker = Keypair.generate();
    const cleaner = Keypair.generate();

    await Promise.all([
      airdrop(provider, user.publicKey),
      airdrop(provider, attacker.publicKey),
      airdrop(provider, cleaner.publicKey),
    ]);

    const [vaultPda, bump] = deriveVaultPda(program.programId, user.publicKey);

    await program.methods
      .createEphemeralVault(approvedAmount)
      .accountsPartial({
        user: user.publicKey,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    return { user, delegate, attacker, cleaner, vaultPda, bump };
  }

  describe("create_ephemeral_vault", () => {
    it("initializes vault state", async () => {
      const f = await createFixture();
      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);

      assert.strictEqual(
        vault.userWallet.toBase58(),
        f.user.publicKey.toBase58(),
      );
      assert.strictEqual(vault.vaultPda.toBase58(), f.vaultPda.toBase58());
      assert.strictEqual(vault.approvedAmount.toNumber(), 2 * LAMPORTS_PER_SOL);
      assert.strictEqual(vault.isActive, true);
      assert.strictEqual(vault.isPaused, false);
      assert.strictEqual(vault.version, PROGRAM_VERSION);
      assert.strictEqual(vault.bump, f.bump);
      assert.strictEqual(vault.availableAmount.toNumber(), 0);
      assert.strictEqual(vault.totalDeposited.toNumber(), 0);
      assert.isNull(vault.delegateWallet);
    });

    it("rejects invalid approved amount bounds", async () => {
      const user = Keypair.generate();
      await airdrop(provider, user.publicKey);
      const [vaultPda] = deriveVaultPda(program.programId, user.publicKey);

      await expectError(
        program.methods
          .createEphemeralVault(new BN(999_999))
          .accountsPartial({
            user: user.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc(),
        "InvalidApprovedAmount",
      );

      await expectError(
        program.methods
          .createEphemeralVault(new BN("1000000000001"))
          .accountsPartial({
            user: user.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc(),
        "InvalidApprovedAmount",
      );
    });
  });

  describe("approve_delegate + renew_session", () => {
    it("owner approves delegate and non-owner cannot", async () => {
      const f = await createFixture();

      await program.methods
        .approveDelegate(f.delegate.publicKey, null)
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(
        vault.delegateWallet?.toBase58(),
        f.delegate.publicKey.toBase58(),
      );
      assert.isNotNull(vault.sessionExpiresAt);

      await expectError(
        program.methods
          .approveDelegate(f.delegate.publicKey, null)
          .accounts({ user: f.attacker.publicKey, vault: f.vaultPda })
          .signers([f.attacker])
          .rpc(),
        "Unauthorized",
      );
    });

    it("rejects self delegate and caps custom duration", async () => {
      const f = await createFixture();

      await expectError(
        program.methods
          .approveDelegate(f.user.publicKey, null)
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "InvalidDelegate",
      );

      await program.methods
        .approveDelegate(f.delegate.publicKey, new BN(9999))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      const now = Math.floor(Date.now() / 1000);
      const expiresAt = vault.sessionExpiresAt!.toNumber();
      assert.isAtMost(expiresAt, now + SESSION_DURATION_SECONDS + 15);
    });

    it("allows renewal only near expiry", async () => {
      const f = await createFixture();

      await expectError(
        program.methods
          .renewSession()
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "NoActiveSession",
      );

      await program.methods
        .approveDelegate(
          f.delegate.publicKey,
          new BN(RENEWAL_WINDOW_SECONDS + 2),
        )
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await sleep(2500);

      await program.methods
        .renewSession()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      const now = Math.floor(Date.now() / 1000);
      assert.isAtLeast(
        vault.sessionExpiresAt!.toNumber(),
        now + SESSION_DURATION_SECONDS - 15,
      );
    });
  });

  describe("auto_deposit_for_trade + execute_trade", () => {
    it("deposits and executes trade with delegate", async () => {
      const f = await createFixture();

      await program.methods
        .approveDelegate(f.delegate.publicKey, null)
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .autoDepositForTrade(new BN(0.5 * LAMPORTS_PER_SOL))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .executeTrade(new BN(100_000), new BN(1_000_000))
        .accounts({ delegate: f.delegate.publicKey, vault: f.vaultPda })
        .signers([f.delegate])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(vault.tradeCount.toNumber(), 1);
      assert.strictEqual(vault.usedAmount.toNumber(), 1_000_000);
      assert.strictEqual(
        vault.availableAmount.toNumber(),
        0.5 * LAMPORTS_PER_SOL - 100_000,
      );
    });

    it("rejects deposit bounds and over-deposit", async () => {
      const f = await createFixture(new BN(2 * LAMPORTS_PER_SOL));

      await expectError(
        program.methods
          .autoDepositForTrade(new BN(999_999))
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "DepositTooSmall",
      );

      await expectError(
        program.methods
          .autoDepositForTrade(MAX_DEPOSIT_AMOUNT.add(new BN(1)))
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "DepositTooLarge",
      );

      await program.methods
        .autoDepositForTrade(new BN(2 * LAMPORTS_PER_SOL))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await expectError(
        program.methods
          .autoDepositForTrade(MIN_DEPOSIT_AMOUNT)
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "OverDeposit",
      );
    });

    it("rejects wrong delegate and auto-revokes expired session", async () => {
      const f = await createFixture();

      await program.methods
        .approveDelegate(f.delegate.publicKey, new BN(3)) // 3 seconds
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .autoDepositForTrade(new BN(MIN_DEPOSIT_AMOUNT))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await expectError(
        program.methods
          .executeTrade(new BN(1000), new BN(1000))
          .accounts({ delegate: f.attacker.publicKey, vault: f.vaultPda })
          .signers([f.attacker])
          .rpc(),
        "Unauthorized",
      );

      // Wait for expiry
      await sleep(3500);

      // Now correct delegate after expiry
      await expectError(
        program.methods
          .executeTrade(new BN(1000), new BN(1000))
          .accounts({ delegate: f.delegate.publicKey, vault: f.vaultPda })
          .signers([f.delegate])
          .rpc(),
        "SessionExpired",
      );

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);

      assert.equal(vault.delegateWallet, null);
      assert.equal(vault.sessionExpiresAt, null);
    });
  });

  describe("withdraw / pause / unpause", () => {
    it("owner withdraws and non-owner is blocked", async () => {
      const f = await createFixture();

      await program.methods
        .autoDepositForTrade(new BN(0.2 * LAMPORTS_PER_SOL))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .withdrawBalance(new BN(0.1 * LAMPORTS_PER_SOL))
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(
        vault.availableAmount.toNumber(),
        0.1 * LAMPORTS_PER_SOL,
      );

      await expectError(
        program.methods
          .withdrawBalance(new BN(1))
          .accounts({ user: f.attacker.publicKey, vault: f.vaultPda })
          .signers([f.attacker])
          .rpc(),
        "Unauthorized",
      );
    });

    it("pause blocks mutating actions until unpaused", async () => {
      const f = await createFixture();

      await program.methods
        .emergencyPause()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await expectError(
        program.methods
          .autoDepositForTrade(MIN_DEPOSIT_AMOUNT)
          .accounts({ user: f.user.publicKey, vault: f.vaultPda })
          .signers([f.user])
          .rpc(),
        "VaultPaused",
      );

      await program.methods
        .unpauseVault()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .autoDepositForTrade(MIN_DEPOSIT_AMOUNT)
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      const vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(vault.isPaused, false);
      assert.isAtLeast(
        vault.totalDeposited.toNumber(),
        MIN_DEPOSIT_AMOUNT.toNumber(),
      );
    });
  });

  describe("revoke / reactivate / cleanup / stats", () => {
    it("revoke deactivates vault and reactivate resets session fields", async () => {
      const f = await createFixture();

      await program.methods
        .approveDelegate(f.delegate.publicKey, null)
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await program.methods
        .revokeAccess()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      let vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(vault.isActive, false);
      assert.isNull(vault.delegateWallet);

      await program.methods
        .reactivateVault()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      vault = await program.account.ephemeralVault.fetch(f.vaultPda);
      assert.strictEqual(vault.isActive, true);
      assert.strictEqual(vault.isPaused, false);
      assert.isNull(vault.delegateWallet);
      assert.isNull(vault.sessionExpiresAt);
    });

    it("cleanup rejects active vault and non-expired inactive vault", async () => {
      const f = await createFixture();

      await expectError(
        program.methods
          .cleanupVault()
          .accounts({
            vault: f.vaultPda,
            userWallet: f.user.publicKey,
            cleaner: f.cleaner.publicKey,
          })
          .signers([f.cleaner])
          .rpc(),
        "VaultStillActive",
      );

      await program.methods
        .revokeAccess()
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      await expectError(
        program.methods
          .cleanupVault()
          .accounts({
            vault: f.vaultPda,
            userWallet: f.user.publicKey,
            cleaner: f.cleaner.publicKey,
          })
          .signers([f.cleaner])
          .rpc(),
        "SessionNotExpired",
      );
    });

    it("get_vault_stats returns session states", async () => {
      const f = await createFixture();

      let stats = await program.methods
        .getVaultStats()
        .accounts({ vault: f.vaultPda })
        .view();

      assert.isDefined(stats.sessionStatus.noSession);

      await program.methods
        .approveDelegate(f.delegate.publicKey, null)
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();

      stats = await program.methods
        .getVaultStats()
        .accounts({ vault: f.vaultPda })
        .view();
      assert.isDefined(stats.sessionStatus.active);

      await program.methods
        .approveDelegate(
          f.delegate.publicKey,
          new BN(RENEWAL_WINDOW_SECONDS + 2),
        )
        .accounts({ user: f.user.publicKey, vault: f.vaultPda })
        .signers([f.user])
        .rpc();
      await sleep(2500);

      stats = await program.methods
        .getVaultStats()
        .accounts({ vault: f.vaultPda })
        .view();
      assert.isDefined(stats.sessionStatus.expiringSoon);
    });
  });
});
