import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Rebalancing, IDL } from "../target/types/rebalancing";
// import type { Rebalancing } from "../target/types/rebalancing";
// import { IDL } from "../target/types/rebalancing";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
	TOKEN_PROGRAM_ID,
	createMint,
	createAccount,
	mintTo,
	getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

// export const IDL = {
// 	version: "0.1.0",
// 	name: "rebalancing",
// 	address: "BUVN6ZFmAigxoKDfsrSbWxNvrbTqNTgdKDuTLibjztBY",
// 	metadata: {
// 		address: "BUVN6ZFmAigxoKDfsrSbWxNvrbTqNTgdKDuTLibjztBY",
// 		version: "0.1.0",
// 		name: "rebalancing",
// 		spec: "0.1.0",
// 	},
// 	instructions: [
// 		{
// 			name: "initialize",
// 			discriminator: [255, 255, 255, 255, 0, 0, 0, 0],
// 			accounts: [
// 				{
// 					name: "config",
// 					isMut: true,
// 					isSigner: true,
// 				},
// 				{
// 					name: "admin",
// 					isMut: true,
// 					isSigner: true,
// 				},
// 				{
// 					name: "systemProgram",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 			],
// 			args: [],
// 		},
// 		{
// 			name: "rebalance",
// 			discriminator: [255, 255, 255, 255, 0, 0, 0, 1],
// 			accounts: [
// 				{
// 					name: "config",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "admin",
// 					isMut: true,
// 					isSigner: true,
// 				},
// 				{
// 					name: "ammProgram",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 				{
// 					name: "amm",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "ammAuthority",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 				{
// 					name: "ammOpenOrders",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "ammCoinVault",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "ammPcVault",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketProgram",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 				{
// 					name: "market",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketBids",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketAsks",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketEventQueue",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketCoinVault",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketPcVault",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "marketVaultSigner",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 				{
// 					name: "userSourceOwner",
// 					isMut: false,
// 					isSigner: true,
// 				},
// 				{
// 					name: "userTokenSource",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "userTokenDestination",
// 					isMut: true,
// 					isSigner: false,
// 				},
// 				{
// 					name: "tokenProgram",
// 					isMut: false,
// 					isSigner: false,
// 				},
// 			],
// 			args: [
// 				{
// 					name: "tokenMints",
// 					type: {
// 						vec: "pubkey",
// 					},
// 				},
// 				{
// 					name: "targetWeights",
// 					type: {
// 						vec: "u8",
// 					},
// 				},
// 			],
// 		},
// 	],
// 	accounts: [
// 		{
// 			name: "rebalanceConfig",
// 			type: {
// 				kind: "struct",
// 				fields: [
// 					{
// 						name: "admin",
// 						type: "publicKey",
// 					},
// 					{
// 						name: "rebalancingActive",
// 						type: "bool",
// 					},
// 				],
// 			},
// 		},
// 	],
// 	errors: [
// 		{
// 			code: 6000,
// 			name: "RebalancingInProgress",
// 			msg: "Rebalancing is currently in progress",
// 		},
// 		{
// 			code: 6001,
// 			name: "InvalidInputLength",
// 			msg: "Invalid input length",
// 		},
// 		{
// 			code: 6002,
// 			name: "InvalidWeights",
// 			msg: "Weights must sum to 100",
// 		},
// 		{
// 			code: 6003,
// 			name: "SwapFailed",
// 			msg: "Swap execution failed",
// 		},
// 	],
// };

describe("rebalancing", () => {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const programId = new PublicKey(
		"2RNkDWBs3fCeA6r6aAA9fBfAx6A4fgQ8V24mutQkZyEt"
	);

	let program: Program<Rebalancing>;

	before(async () => {
		console.log("before");
		program = await Program.at<Rebalancing>(programId, provider);
		console.log("after");
	});

	// We'll need these keypairs
	const admin = Keypair.generate();
	const user = provider.wallet.publicKey;

	// Token mints
	let usdcMint: PublicKey;
	let usdtMint: PublicKey;

	// Token accounts
	let userUsdcAccount: PublicKey;
	let userUsdtAccount: PublicKey;
	let vaultUsdcAccount: PublicKey;
	let vaultUsdtAccount: PublicKey;

	// Program derived address for config
	let configPDA: PublicKey;

	it("Initialize program", async () => {
		// Airdrop SOL to admin
		const signature = await provider.connection.requestAirdrop(
			admin.publicKey,
			1000000000
		);
		await provider.connection.confirmTransaction(signature);

		// Find config PDA
		[configPDA] = PublicKey.findProgramAddressSync(
			[Buffer.from("config")],
			program.programId
		);

		// Initialize program
		await program.methods
			.initialize()
			.accounts({
				config: configPDA,
				admin: admin.publicKey,
				systemProgram: SystemProgram.programId,
			})
			.signers([admin])
			.rpc({});
	});

	it("Set up token accounts and initial balances", async () => {
		// Create USDC mint
		usdcMint = await createMint(
			provider.connection,
			admin,
			admin.publicKey,
			null,
			6
		);

		// Create USDT mint
		usdtMint = await createMint(
			provider.connection,
			admin,
			admin.publicKey,
			null,
			6
		);

		// Create user token accounts
		userUsdcAccount = await createAccount(
			provider.connection,
			admin,
			usdcMint,
			user
		);

		userUsdtAccount = await createAccount(
			provider.connection,
			admin,
			usdtMint,
			user
		);

		// Create vault token accounts
		vaultUsdcAccount = await createAccount(
			provider.connection,
			admin,
			usdcMint,
			configPDA
		);

		vaultUsdtAccount = await createAccount(
			provider.connection,
			admin,
			usdtMint,
			configPDA
		);

		// Mint initial tokens to user
		await mintTo(
			provider.connection,
			admin,
			usdcMint,
			userUsdcAccount,
			admin,
			75_000_000 // 75 USDC
		);

		await mintTo(
			provider.connection,
			admin,
			usdtMint,
			userUsdtAccount,
			admin,
			25_000_000 // 25 USDT
		);

		// Verify initial balances
		const usdcAccount = await getAccount(
			provider.connection,
			userUsdcAccount
		);
		const usdtAccount = await getAccount(
			provider.connection,
			userUsdtAccount
		);

		assert.equal(usdcAccount.amount.toString(), "75000000");
		assert.equal(usdtAccount.amount.toString(), "25000000");
	});

	it("Execute rebalance", async () => {
		await program.methods
			.rebalance(
				[usdcMint, usdtMint], // token mints
				[50, 50] // target weights (50/50 split)
			)
			.accounts({
				config: configPDA,
				admin: admin.publicKey,
				tokenProgram: TOKEN_PROGRAM_ID,
			})
			.remainingAccounts([
				{ pubkey: vaultUsdcAccount, isWritable: true, isSigner: false },
				{ pubkey: vaultUsdtAccount, isWritable: true, isSigner: false },
			])
			.signers([admin])
			.rpc();

		// Verify final balances
		const usdcAccount = await getAccount(
			provider.connection,
			vaultUsdcAccount
		);
		const usdtAccount = await getAccount(
			provider.connection,
			vaultUsdtAccount
		);

		// Should be roughly 50/50 split (allowing for some slippage)
		const usdcBalance = Number(usdcAccount.amount);
		const usdtBalance = Number(usdtAccount.amount);
		const total = usdcBalance + usdtBalance;

		const usdcPercentage = (usdcBalance / total) * 100;
		const usdtPercentage = (usdtBalance / total) * 100;

		assert(
			Math.abs(usdcPercentage - 50) < 2,
			"USDC balance should be close to 50%"
		);
		assert(
			Math.abs(usdtPercentage - 50) < 2,
			"USDT balance should be close to 50%"
		);
	});
});
