import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { NftShop } from "../target/types/nft_shop";
import {
  TOKEN_PROGRAM_ID,
  createInitializeMintInstruction,
  MINT_SIZE,
  createMintToInstruction,
  createInitializeAccountInstruction,
  ACCOUNT_SIZE,
  createTransferInstruction,
} from "@solana/spl-token";

const PROGRAM_ID = (anchor.workspace.NftShop as Program<NftShop>).programId;
export const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const METADATA_PREFIX = "metadata";
const EDITION = "edition";
const VAULT_OWNER_PREFIX = "mt_vault";
const HISTORY_PREFIX = "history";
const PAYOUT_TICKET_PREFIX = "payout_ticket";
const HOLDER_PREFIX = "holder";
const PRIMARY_METADATA_CREATORS_PREFIX = "primary_creators";
const EDITION_MARKER_BIT_SIZE = 248;
const HUNDRED_SOL = 100_000_000_000;
const EMPTY_SPACE = 0;

export const createSystemAccount = async ({
  provider,
  payer,
}: {
  provider: anchor.Provider;
  payer: anchor.Wallet;
}): Promise<anchor.web3.Keypair> => {
  const newAccountKeypair = anchor.web3.Keypair.generate();
  const space = EMPTY_SPACE;
  const lamports = await provider.connection.getMinimumBalanceForRentExemption(
    space
  );

  // Create a System Account with 100 SOL balance
  const tx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: newAccountKeypair.publicKey,
      lamports,
      space,
      programId: anchor.web3.SystemProgram.programId,
    }),
    anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: newAccountKeypair.publicKey,
      lamports: HUNDRED_SOL,
    })
  );

  await provider.sendAndConfirm(tx, [payer.payer, newAccountKeypair]);
  return newAccountKeypair;
};

export const createMintAccount = async ({
  provider,
  payer,
  mint,
  decimals,
  mintAuthority,
  freezeAuthority,
}: {
  provider: anchor.Provider;
  payer: anchor.Wallet;
  mint: anchor.web3.Keypair;
  decimals?: number;
  mintAuthority?: anchor.web3.PublicKey;
  freezeAuthority?: anchor.web3.PublicKey;
}): Promise<string> => {
  const lamports: number =
    await provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);

  const tx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: mintAuthority ?? payer.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID,
      lamports,
    }),

    createInitializeMintInstruction(
      mint.publicKey,
      decimals ?? 0,
      mintAuthority ?? payer.publicKey,
      freezeAuthority ?? payer.publicKey
    )
  );

  return await provider.sendAndConfirm(tx, [mint]);
};

export const createTokenAccount = async ({
  provider,
  payer,
  tokenAccount,
  mint,
  owner,
}: {
  provider: anchor.Provider;
  payer: anchor.Wallet;
  tokenAccount: anchor.web3.Keypair;
  mint: anchor.web3.PublicKey;
  owner?: anchor.web3.PublicKey;
}): Promise<string> => {
  const lamports: number =
    await provider.connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE);

  const tx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: tokenAccount.publicKey,
      space: ACCOUNT_SIZE,
      programId: TOKEN_PROGRAM_ID,
      lamports,
    }),
    createInitializeAccountInstruction(
      tokenAccount.publicKey,
      mint,
      owner ?? payer.publicKey
    )
  );

  return await provider.sendAndConfirm(tx, [tokenAccount]);
};

export const mintTo = async ({
  provider,
  mint,
  destination,
  authority,
  amount,
}: {
  provider: anchor.Provider;
  mint: anchor.web3.PublicKey;
  destination: anchor.web3.PublicKey;
  authority: anchor.web3.Keypair;
  amount: number;
}): Promise<string> => {
  const tx = new anchor.web3.Transaction().add(
    createMintToInstruction(mint, destination, authority.publicKey, amount)
  );

  return await provider.sendAndConfirm(tx, [authority]);
};

export const findVaultOwnerAddress = (
  mint: anchor.web3.PublicKey,
  store: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_OWNER_PREFIX), mint.toBuffer(), store.toBuffer()],
    PROGRAM_ID
  );

export const findTreasuryOwnerAddress = (
  treasuryMint: anchor.web3.PublicKey,
  sellingResource: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(HOLDER_PREFIX),
      treasuryMint.toBuffer(),
      sellingResource.toBuffer(),
    ],
    PROGRAM_ID
  );

export const findTradeHistoryAddress = (
  wallet: anchor.web3.PublicKey,
  market: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(HISTORY_PREFIX), wallet.toBuffer(), market.toBuffer()],
    PROGRAM_ID
  );

export const findPrimaryMetadataCreatorsAddress = (
  metadata: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(PRIMARY_METADATA_CREATORS_PREFIX), metadata.toBuffer()],
    PROGRAM_ID
  );

export const findPayoutTicketAddress = (
  market: anchor.web3.PublicKey,
  funder: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(PAYOUT_TICKET_PREFIX), market.toBuffer(), funder.toBuffer()],
    PROGRAM_ID
  );
};

export const findMetadataAddress = ({
  mint,
}: {
  mint: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_PREFIX),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

export const findMasterEditionAddress = ({
  mint,
}: {
  mint: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_PREFIX),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(EDITION),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

export const findEditionMarkerAddress = ({
  mint,
  supply,
}: {
  mint: anchor.web3.PublicKey;
  supply: number;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_PREFIX),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(EDITION),
      Buffer.from(
        new BN(Math.floor(supply / EDITION_MARKER_BIT_SIZE)).toString()
      ),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
