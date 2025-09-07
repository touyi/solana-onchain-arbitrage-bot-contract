import * as anchor from "@coral-xyz/anchor";
import * as solana from "@solana/web3.js";
import type { GetVersionedTransactionConfig } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { ArbTouyi } from "../target/types/arb_touyi";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

describe("arb_touyi", () => {
  console.log("in test")
  const mainnetConnection = new solana.Connection("{RPC_URL_HERE}");
  const provider = new anchor.AnchorProvider(mainnetConnection, anchor.AnchorProvider.env().wallet);
  anchor.setProvider(provider);

  const program = anchor.workspace.ArbTouyi as Program<ArbTouyi>;

  it("Is initialized!", async () => {
    // Add your test here.
    try {
      
      const tx = await program.methods.testSwap().accounts({
        lbPair: new solana.PublicKey("71HuFmuYAFEFUna2x2R4HJjrFNQHGuagW3gUMFToL9tk"),
        binArrayBitmapExtension: null,
        reserveX: new solana.PublicKey("Hp5K2KWRoF2LXDsYQaoE18VheQKPrsLQ9dzNELzPULZb"),
        reserveY: new solana.PublicKey("71KtbjVeGBbB8VAsniAoFw59sZ2EWgwqj3r1rLLccL59"),
        userTokenIn: new solana.PublicKey("Gf3kKUoaMaQ5B1UyJaeqa9mbpR6i8YvYXUk3b8hZ3HSD"),
        userTokenOut: new solana.PublicKey("HzH9mrNgsUiitXqJjTcrSfqySfS4652vKCDQRNofTSH8"),
        tokenXMint: new solana.PublicKey("6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN"),
        tokenYMint: new solana.PublicKey("So11111111111111111111111111111111111111112"),
        oracle: new solana.PublicKey("7fb3hkzhroueZsxWtYnAfmMZ9o9RfMPoX6uWu4vFaKxt"),
        hostFeeIn: null,
        user: provider.wallet.publicKey,
        tokenXProgram: new solana.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        tokenYProgram: new solana.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        eventAuthority: new solana.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        binArrays: new solana.PublicKey("6QUU34JLRG9dqjY7NqktkWWHcddsakp6hFXrX3FeApne"),
      }).rpc({
        commitment: 'confirmed'
      })
      const rex = await anchor.getProvider().connection.getTransaction(tx, {
        commitment: 'confirmed',
        maxSupportedTransactionVersion:0
      })
      console.log("Your transaction signature", tx, rex);
    } catch (error) {
      console.log("error", error);
    }
  });
});
