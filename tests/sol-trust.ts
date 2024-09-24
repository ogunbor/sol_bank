import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolTrust } from "../target/types/sol_trust";
import { BankRewards } from "../target/types/bank_rewards";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";


const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor
    .getProvider()
    .connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    "confirmed"
  );
  return signature;
};

describe("sol-trust", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolTrust as Program<SolTrust>;
  const provider = anchor.getProvider();
  const connection = provider.connection;

  // Create a new user keypair
  const user = new Keypair();
  
  // Create a new admin keypair
  const admin = new Keypair();

  // Find program addresses for state and vault
  const [state, stateBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("state"), user.publicKey.toBytes()],
    program.programId
  );

  const [vault, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), state.toBytes()],
    program.programId
  );

  it("Airdrop SOL to user and admin", async () => {
    const userAirdropTx = await connection.requestAirdrop(
      user.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );
    await confirmTx(userAirdropTx);

    const adminAirdropTx = await connection.requestAirdrop(
      admin.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL 
    );
    await confirmTx(adminAirdropTx);
  });

  it("Initialize", async () => {
    try {
      const lockDuration = new BN(0); 

      const tx = await program.methods
        .initialize(lockDuration)
        .accountsStrict({
          user: user.publicKey,
          vaultState: state,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc()
        .then(confirmTx);

      console.log("Your transaction signature", tx);
    } catch (e) {
      console.error(e);
      throw e;
    }
  });

  it("Deposit into Vault", async () => {
    const depositAmount = new BN(5 * anchor.web3.LAMPORTS_PER_SOL); // Deposit 5 SOL
    try {
      const tx = await program.methods
        .deposit(depositAmount)
        .accountsStrict({
          user: user.publicKey,
          vaultState: state,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc()
        .then(confirmTx);
      console.log("Deposited successfully with transaction signature:", tx);
    } catch (e) {
      console.error("Deposit failed:", e);
      throw e;
    }
  });

  //  it("PrematureClose", async () => {
  //   try {
  //     const tx = await program.methods
  //       .prematureClose() // Make sure this method is properly defined in your Anchor program
  //       .accountsStrict({
  //         user: user.publicKey,
  //         vault: vault,
  //         vaultState: state,
  //         adminWallet: admin.publicKey, // Pass the admin wallet public key
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([user])
  //       .rpc({ skipPreflight: true })
  //       .then(confirmTx);

  //     console.log("Your transaction signature", tx);
  //   } catch (e) {
  //     console.error//(e);
  //     throw e;
  //   }
  // });

it("MatureClose", async () => {
  try {
    const bankRewardsProgramId = new PublicKey("BBoAqxz7AfBvtkDgj2XtjG9kMmUEciSg6xmyLCJmzNGY"); 
    
    const tx = await program.methods
      .matureClose()
      .accountsStrict({
        user: user.publicKey,
        vault: vault,
        vaultState: state,
        bankVault: vault, 
        bankVaultState: state, 
        bankRewardsProgram: bankRewardsProgramId, 
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Mature close executed with transaction signature", tx);
  } catch (e) {
    console.error(e);
    throw e;
  }
});

 
});



