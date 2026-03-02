import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaGungzCustomVault } from "../target/types/solana_gungz_custom_vault";
import { expect } from "chai";

describe("solana-gungz-custom-vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const owner = provider.wallet.publicKey;
  const program = anchor.workspace.SolanaGungzCustomVault as Program<SolanaGungzCustomVault>;

  const [vaultStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), owner.toBuffer()],
    program.programId
  );

  const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultStatePda.toBuffer()],
    program.programId
  );

  let member: anchor.web3.Keypair;
  let memberAccountPda: anchor.web3.PublicKey;

  before(async () => {
    await provider.connection.requestAirdrop(owner, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  it("Initialize the vault", async () => {
    await program.methods
      .initialize()
      .accountsStrict({
        user: owner,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    expect(vaultState.owner.toString()).to.equal(owner.toString());
    expect(vaultState.memberCount).to.equal(0);
  });

  it("Deposit SOL into the vault", async () => {
    const depositAmount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);

    await program.methods
      .deposit(depositAmount)
      .accountsStrict({
        user: owner,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    expect(finalVaultBalance).to.equal(initialVaultBalance + depositAmount.toNumber());
  });

  it("Withdraw SOL from the vault", async () => {
    const withdrawAmount = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);

    await program.methods
      .withdraw(withdrawAmount)
      .accountsStrict({
        user: owner,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    expect(finalVaultBalance).to.equal(initialVaultBalance - withdrawAmount.toNumber());
  });

  it("Add member to vault", async () => {
    member = anchor.web3.Keypair.generate();
    await provider.connection.requestAirdrop(member.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise((resolve) => setTimeout(resolve, 1000));

    [memberAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("member_state"), vaultStatePda.toBuffer(), member.publicKey.toBuffer()],
      program.programId
    );

    const limitAmount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
    const unlockTs = new anchor.BN(Math.floor(Date.now() / 1000) + 10);

    await program.methods
      .addMember(limitAmount, unlockTs)
      .accountsStrict({
        vaultState: vaultStatePda,
        memberAccount: memberAccountPda,
        userToAdd: member.publicKey,
        owner: owner,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const memberAccount = await program.account.memberAccount.fetch(memberAccountPda);
    expect(memberAccount.user.toString()).to.equal(member.publicKey.toString());
    expect(memberAccount.limitAmount.toNumber()).to.equal(limitAmount.toNumber());

    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    expect(vaultState.memberCount).to.equal(1);
  });

  it("Member withdraw fails when locked", async () => {
    const withdrawAmount = new anchor.BN(0.05 * anchor.web3.LAMPORTS_PER_SOL);

    try {
      await program.methods
        .memberWithdraw(withdrawAmount)
        .accountsStrict({
          memberAccount: memberAccountPda,
          vaultState: vaultStatePda,
          member: member.publicKey,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([member])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("StillLocked");
    }
  });

  it("Member withdraw succeeds after unlock", async () => {
    await new Promise((resolve) => setTimeout(resolve, 11000));

    const withdrawAmount = new anchor.BN(0.05 * anchor.web3.LAMPORTS_PER_SOL);
    const initialMemberBalance = await provider.connection.getBalance(member.publicKey);

    await program.methods
      .memberWithdraw(withdrawAmount)
      .accountsStrict({
        memberAccount: memberAccountPda,
        vaultState: vaultStatePda,
        member: member.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([member])
      .rpc();

    const finalMemberBalance = await provider.connection.getBalance(member.publicKey);
    expect(finalMemberBalance).to.be.greaterThan(initialMemberBalance);

    const memberAccount = await program.account.memberAccount.fetch(memberAccountPda);
    expect(memberAccount.amountWithdrawn.toNumber()).to.equal(withdrawAmount.toNumber());
  });

  it("Member withdraw fails when exceeds limit", async () => {
    const withdrawAmount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);

    try {
      await program.methods
        .memberWithdraw(withdrawAmount)
        .accountsStrict({
          memberAccount: memberAccountPda,
          vaultState: vaultStatePda,
          member: member.publicKey,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([member])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("ExceedsLimit");
    }
  });

  it("Remove member from vault", async () => {
    await program.methods
      .removeMember()
      .accountsStrict({
        vaultState: vaultStatePda,
        memberAccount: memberAccountPda,
        userToRemove: member.publicKey,
        owner: owner,
      })
      .rpc();

    const memberAccountInfo = await provider.connection.getAccountInfo(memberAccountPda);
    expect(memberAccountInfo).to.be.null;

    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    expect(vaultState.memberCount).to.equal(0);
  });

  it("Close fails with members", async () => {
    const member2 = anchor.web3.Keypair.generate();
    const [member2AccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("member_state"), vaultStatePda.toBuffer(), member2.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addMember(new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL), new anchor.BN(Math.floor(Date.now() / 1000) + 10))
      .accountsStrict({
        vaultState: vaultStatePda,
        memberAccount: member2AccountPda,
        userToAdd: member2.publicKey,
        owner: owner,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    try {
      await program.methods
        .close()
        .accountsStrict({
          user: owner,
          vaultState: vaultStatePda,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("MembersExist");
    }

    await program.methods
      .removeMember()
      .accountsStrict({
        vaultState: vaultStatePda,
        memberAccount: member2AccountPda,
        userToRemove: member2.publicKey,
        owner: owner,
      })
      .rpc();
  });

  it("Close the vault", async () => {
    await program.methods
      .close()
      .accountsStrict({
        user: owner,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const vaultStateInfo = await provider.connection.getAccountInfo(vaultStatePda);
    expect(vaultStateInfo).to.be.null;
  });
});
