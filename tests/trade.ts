import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Trade } from "../target/types/trade";
import { SystemProgram, Keypair, PublicKey, Transaction, SYSVAR_RENT_PUBKEY, SYSVAR_CLOCK_PUBKEY, Connection } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createAssociatedTokenAccount, getAssociatedTokenAddress , ASSOCIATED_TOKEN_PROGRAM_ID,createMint, mintTo, mintToChecked, getAccount, getMint, getAssociatedTokenAddressSync,  } from "@solana/spl-token";

describe("trade", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Trade as Program<Trade>;

  let globalState, vault, tokenVaultAccount: PublicKey;

  let globalStateBump, vaultBump, tokenVaultAccountBump: number;

  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const TOKEN_VAULT_SEED = "TOKEN-VAULT-SEED";
  const VAULT_SEED = "VAULT-SEED";

  const priceSolFeed = new anchor.web3.PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");
  const tokenMint = new PublicKey("8NtheYSKWDkCgWoc8HScQFkcCTF1FiFEbbriosZLNmtE");

  let user = Keypair.fromSecretKey(Uint8Array.from(/* Uint8Array of user wallet*/));

  let owner = Keypair.fromSecretKey(
    Uint8Array.from(/* Uint8Array of user wallet*/)
  );

  type Event = anchor.IdlEvents<typeof program["idl"]>;

  it("Get PDA Accounts", async() => {
    [globalState, globalStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(GLOBAL_STATE_SEED)
      ],
      program.programId
    );

    [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(VAULT_SEED)
      ],
      program.programId
    );

    [tokenVaultAccount, tokenVaultAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(TOKEN_VAULT_SEED),
        tokenMint.toBuffer()
      ],
      program.programId
    );
  });

  
  it("Is initialized!", async () => {
    try {
      const tx = await program.rpc.initialize(
        {
          accounts: {
            owner: owner.publicKey,
            globalState,
            vault,
            tokenMint,
            tokenVaultAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
          },
          signers: [owner]
        }
      );
      const globalStateData = await program.account.globalState.fetch(globalState);
      console.log("globalStateData->", globalStateData);
    } catch (error) {
      console.log(error);
    }
  });
  
  it("Deposit 1 SOL to vault", async() => {
    try {
      let depositAmount = 100000000;

      const tx = await program.rpc.depositSolVault(
        new anchor.BN(depositAmount),
        {
          accounts: {
            owner: owner.publicKey,
            globalState,
            vault,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );

      const balance = await program.provider.connection.getBalance(vault);
      console.log("vault balance->", Number(balance));

      const globalStateData = await program.account.globalState.fetch(globalState);
      console.log("globalStateData Sol Balance->", Number(globalStateData.solBalance));
    } catch (error) {
      console.log(error);
    }
    

  });


  it("Deposit 1000 USDC to token vault account", async() => {
    try {
      const tokenOwnerAccount = await getAssociatedTokenAddress(
        tokenMint,
        owner.publicKey
      );

      const depositAmount = 1000000000;

      const tx = await program.rpc.depositUsdcVault(
        new anchor.BN(depositAmount),
        {
          accounts: {
            owner: owner.publicKey,
            globalState,
            tokenMint,
            tokenVaultAccount,
            tokenOwnerAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
          },
          signers: [owner]
        }
      );

      const tokenBalance = await getTokenBalanceSpl(program.provider.connection, tokenVaultAccount);
      console.log("tokenBalance->", tokenBalance);

      const globalStateData = await program.account.globalState.fetch(globalState);
      console.log("globalStateData Usdc Balance->", Number(globalStateData.usdcBalance));
    } catch (error) {
      console.log(error);
    }
  });
 


  it("Trade USDC with 1 SOL", async() => {
    try {
      const tradeSolAmount = 1000000000; // 1SOL
      const tokenOwnerAccount = await getAssociatedTokenAddress(
        tokenMint,
        user.publicKey,
      )
      let listenerId: number;
      const event = await new Promise<Event[E]>(async (res) => {
        listenerId = program.addEventListener("TradeUSDCWithSolEvent", (event) => {
          res(event);
        });
        const tx = await program.rpc.buyUsdcWithSol(
          new anchor.BN(tradeSolAmount),
          {
            accounts: {
              user: user.publicKey,
              globalState,
              vault,
              tokenMint,
              tokenVaultAccount,
              tokenOwnerAccount,
              priceSolFeed,
              systemProgram: SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              clock: SYSVAR_CLOCK_PUBKEY
            },
            signers: [user]
          }
        )
      });
      await program.removeEventListener(listenerId);
      console.log("event->",event);
    } catch (error) {
      console.log(error)
    }
  });

  it("Trade Sol with 100 USDC", async() => {
    try {
      const tradeSolAmount = 100000000; // 100 USDC
      const tokenOwnerAccount = await getAssociatedTokenAddress(
        tokenMint,
        user.publicKey,
      )
      let listenerId: number;
      const event = await new Promise<Event[E]>(async (res) => {
        listenerId = program.addEventListener("TradeSolWithUSDCEvent", (event) => {
          res(event);
        });
        const tx = await program.rpc.buySolWithUsdc(
          new anchor.BN(tradeSolAmount),
          {
            accounts: {
              user: user.publicKey,
              globalState,
              vault,
              tokenMint,
              tokenVaultAccount,
              tokenOwnerAccount,
              priceSolFeed,
              systemProgram: SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              clock: SYSVAR_CLOCK_PUBKEY
            },
            signers: [user]
          }
        )
      });
      await program.removeEventListener(listenerId);
      console.log("event->",event);
    } catch (error) {
      console.log(error)
    }
  })
});



async function getTokenBalanceSpl(connection: Connection, tokenAccount: PublicKey) {
  const info = await getAccount(connection, tokenAccount);
  const amount = Number(info.amount);
  const mint = await getMint(connection, info.mint);
  // const balance = amount / (10 ** mint.decimals);
  const balance = amount;
  return balance;
}

