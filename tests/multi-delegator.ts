import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { MultiDelegator } from "../target/types/multi_delegator";
import { Account, createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import { setup } from "mocha";

const { LAMPORTS_PER_SOL } = anchor.web3;

describe("multi-delegator", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MultiDelegator as Program<MultiDelegator>;

  const connection = anchor.AnchorProvider.env().connection;
  const wallet = anchor.web3.Keypair.generate();
  
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  const merchant = anchor.web3.Keypair.generate();
  const merchant2 = anchor.web3.Keypair.generate();

  let user1ATA: Account;
  let user2ATA: Account;
  let merchantATA: Account;
  let merchant2ATA: Account;
  let mint: anchor.web3.PublicKey;

  setup(async () => {

    // Airdrop SOL to the wallet
    console.log("Balance: ", await connection.getBalance(wallet.publicKey));
    const tx = await connection.requestAirdrop(wallet.publicKey, 1000 * LAMPORTS_PER_SOL);
    const tx2 = await connection.requestAirdrop(user1.publicKey, 1000 * LAMPORTS_PER_SOL);
    const tx3 = await connection.requestAirdrop(user2.publicKey, 1000 * LAMPORTS_PER_SOL);
    const tx4 = await connection.requestAirdrop(merchant.publicKey, 1000 * LAMPORTS_PER_SOL);
    const tx5 = await connection.requestAirdrop(merchant2.publicKey, 1000 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(tx);
    await connection.confirmTransaction(tx2);
    await connection.confirmTransaction(tx3);
    await connection.confirmTransaction(tx4);
    await connection.confirmTransaction(tx5);

    console.log("Airdropped SOL to wallet: ", tx);
    
    mint = await createMint(
      connection,
      wallet,
      wallet.publicKey,
      null,
      6,
      undefined,
      { commitment: "confirmed" }
    );
    
    console.log("mint: ", mint);

    user1ATA = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, user1.publicKey);
    user2ATA = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, user2.publicKey);
    merchantATA = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, merchant.publicKey);
    merchant2ATA = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, merchant2.publicKey);

    await mintTo(connection, wallet, mint, user1ATA.address, wallet.publicKey, 1000);
  });

  it("simple delegation test", async () => {
    
    // derive delegation pda key
    const [delegationPdaKey, delegationPdaBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("delegation"), user1ATA.address.toBuffer(), user2.publicKey.toBuffer(), new BN(1).toArrayLike(Buffer, "le")],
      program.programId
    );

    // call add delegate on user1ata
    await program.methods.addDelegate({simple: [new BN(1000)]}).accounts({
      authority: user1.publicKey,
      delegate: user2.publicKey,
      tokenAccount: user1ATA.address,
      delegation: delegationPdaKey,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([user1]).rpc();

    const delegationAccountBefore = await program.account.delegation.fetch(delegationPdaKey);
    console.log("delegationAccountBefore: ", delegationAccountBefore.kind.simple[0].toNumber());

    assert(delegationAccountBefore.kind.simple[0].toNumber() === 1000);
    
    await program.methods.transfer(new BN(100)).accounts({
      delegate: user2.publicKey,
      delegation: delegationPdaKey,
      source: user1ATA.address,
      destination: user2ATA.address,
      mint: mint,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([user2]).rpc({skipPreflight: true}); 

    const delegationAccountAfter = await program.account.delegation.fetch(delegationPdaKey);
    console.log("delegationAccountAfter: ", delegationAccountAfter.kind.simple[0].toNumber());

    // get ata2 balance
    const ata2Balance = await connection.getTokenAccountBalance(user2ATA.address);
    console.log("ata2Balance: ", ata2Balance);

    assert(delegationAccountAfter.kind.simple[0].toNumber() === 900);
    assert(ata2Balance.value.amount === "100");

  });

  it("recurring delegation test", async () => {

    // derive delegation for merchant
    const [delegationPdaKey2, delegationPdaBump2] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("delegation"), user1ATA.address.toBuffer(), merchant.publicKey.toBuffer(), new BN(2).toArrayLike(Buffer, "le")],
      program.programId
    );

    
    const start = Math.floor(new Date().getTime() / 1000);

    await program.methods.addDelegate(
      {
        recurring: [
        {
          amountPerPeriod: new BN(1), 
          period: new BN(1), 
          startDate: new BN(start), 
          endDate: new BN(start + 100), 
          lastClaim: new BN(1), 
          destination: merchantATA.address
        }]
      }
      ).accounts({
      authority: user1.publicKey,
      delegate: merchant.publicKey,
      tokenAccount: user1ATA.address,
      delegation: delegationPdaKey2,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([user1]).rpc({skipPreflight: true});

    // get merchant ata balance
    const merchantAtaBalance = await connection.getTokenAccountBalance(merchantATA.address);
    console.log("merchantAtaBalance: ", merchantAtaBalance);
    assert(merchantAtaBalance.value.amount === "0");

    // delegation account 2
    const delegationAccount2Before = await program.account.delegation.fetch(delegationPdaKey2);
    console.log("delegationAccount2Before: ", delegationAccount2Before);
    console.log("delegationAccount2Before: ", delegationAccount2Before.kind.recurring[0]);
    console.log("equal: ", delegationAccount2Before.kind.recurring[0].amountPerPeriod === new BN(1));
    console.log("equal: ", delegationAccount2Before.kind.recurring[0].amountPerPeriod);
    console.log("equal: ", new BN(1));
    assert(delegationAccount2Before.kind.recurring[0].amountPerPeriod.eq(new BN(1)));
    assert(delegationAccount2Before.kind.recurring[0].period.eq(new BN(1)));
    assert(delegationAccount2Before.kind.recurring[0].startDate.eq(new BN(start)));
    assert(delegationAccount2Before.kind.recurring[0].endDate.eq(new BN(start + 100)));
    assert(delegationAccount2Before.kind.recurring[0].lastClaim.eq(new BN(start)));
    assert(delegationAccount2Before.kind.recurring[0].destination.equals(merchantATA.address));

    // sleep for 5 seconds
    await new Promise(resolve => setTimeout(resolve, 5000));

    // transfer recurring
    await program.methods.transferRecurring().accounts({

      delegate: merchant.publicKey,
      delegation: delegationPdaKey2,
      source: user1ATA.address,
      destination: merchantATA.address,
      mint: mint,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc({skipPreflight: true});

    const delegationAccount2After = await program.account.delegation.fetch(delegationPdaKey2);
    console.log("delegationAccount2After: ", delegationAccount2After);
    console.log("delegationAccount2After: ", delegationAccount2After.kind.recurring[0]);
    const periods = delegationAccount2After.kind.recurring[0].lastClaim.sub(new BN(start));
    console.log("periods: ", periods);

    // get updated merchant ata balance
    const merchantAtaBalance2 = await connection.getTokenAccountBalance(merchantATA.address);
    console.log("merchantAtaBalance2: ", merchantAtaBalance2);
    assert(merchantAtaBalance2.value.amount === periods.toString());


  });

  it("recurring delegation test with non-complete period and prorated amounts!", async () => {
    
    // derive delegation for merchant2
    const [delegationPdaKey3, delegationPdaBump3] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("delegation"), user1ATA.address.toBuffer(), merchant2.publicKey.toBuffer(), new BN(2).toArrayLike(Buffer, "le")],
      program.programId
    );

    // get current time as unix timestamp
    const start2 = Math.floor(new Date().getTime() / 1000);

    await program.methods.addDelegate(
      {
        recurring: [
        {
          amountPerPeriod: new BN(100), 
          period: new BN(10), 
          startDate: new BN(start2), 
          endDate: new BN(start2 + 100), 
          lastClaim: new BN(1), 
          destination: merchant2ATA.address
        }]
      }
      ).accounts({
      authority: user1.publicKey,
      delegate: merchant2.publicKey,
      tokenAccount: user1ATA.address,
      delegation: delegationPdaKey3,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([user1]).rpc({skipPreflight: true});

    
    const delegationAccount3Before = await program.account.delegation.fetch(delegationPdaKey3);
    console.log("delegationAccount3Before: ", delegationAccount3Before);
    console.log("delegationAccount3Before: ", delegationAccount3Before.kind.recurring[0]);

    // get merchant 2 balance
    const merchant2AtaBalance = await connection.getTokenAccountBalance(merchant2ATA.address);
    console.log("merchant2AtaBalance: ", merchant2AtaBalance);
    assert(merchant2AtaBalance.value.amount === "0");

    // sleep for 5 seconds
    await new Promise(resolve => setTimeout(resolve, 5000));

    console.log("test");
    // transfer recurring
    await program.methods.transferRecurring().accounts({
      delegate: merchant2.publicKey,
      delegation: delegationPdaKey3,
      source: user1ATA.address,
      destination: merchant2ATA.address,
      mint: mint,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc({skipPreflight: true});
    
    console.log("test2");

    // get merchant 2 post transfer balance
    const merchant2AtaBalance2 = await connection.getTokenAccountBalance(merchant2ATA.address);
    console.log("merchant2AtaBalance2: ", merchant2AtaBalance2);

    // get delegation account 3 post transfer balance
    const delegationAccount3After = await program.account.delegation.fetch(delegationPdaKey3);
    console.log("delegationAccount3After: ", delegationAccount3After);
    console.log("delegationAccount3After: ", delegationAccount3After.kind.recurring[0]);
    const periods2 = delegationAccount3After.kind.recurring[0].lastClaim.sub(new BN(start2));
    console.log("periods2: ", periods2);


    const expectedAmount = periods2.mul(new BN(100)).div(new BN(10));

    assert(merchant2AtaBalance2.value.amount === expectedAmount.toString());




  });
});
