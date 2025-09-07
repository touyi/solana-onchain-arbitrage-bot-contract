"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const anchor = __importStar(require("@coral-xyz/anchor"));
const solana = __importStar(require("@solana/web3.js"));
describe("arb_touyi", () => {
    const mainnetConnection = new solana.Connection("{RPC_URL}");

    const provider = new anchor.AnchorProvider(mainnetConnection, anchor.AnchorProvider.env().wallet);
    anchor.setProvider(provider);
    const program = anchor.workspace.ArbTouyi;
    it("Is initialized!", () => __awaiter(void 0, void 0, void 0, function* () {
        // Add your test here.
        try {
            const tx = yield program.methods.testSwap().accounts({
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
                binArrays: new solana.PublicKey("6QUU34JLRG9dqjY7NqktkWWHcddsakp6hFXrX3FeApne"),
            }).rpc({
                commitment: 'confirmed'
            });
            const rex = yield anchor.getProvider().connection.getTransaction(tx, {
                commitment: 'confirmed',
                maxSupportedTransactionVersion: 0
            });
            console.log("Your transaction signature", tx, rex);
        }
        catch (error) {
            console.log("error", error);
        }
    }));
});
