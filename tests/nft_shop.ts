import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { NftShop } from "../target/types/nft_shop";
import { NftMinter } from "../target/types/nft_minter";
import {
  TOKEN_METADATA_PROGRAM_ID,
  findMetadataAddress,
  findEditionAddress,
  findVaultOwnerAddress,
  createMintAccount,
  createTokenAccount,
  mintTo,
  findPayoutTicketAddress,
  findTreasuryOwnerAddress,
  findTradeHistoryAddress,
  findEditionMarkerAddress,
  findPrimaryMetadataCreatorsAddress,
  createSystemAccount,
} from "./utils";

export const createStore = async ({
  nftShopProgram,
  payer,
}: {
  nftShopProgram: anchor.Program<NftShop>;
  payer: anchor.Wallet;
}): Promise<{
  storeKeypair: anchor.web3.Keypair;
  storeAdminKeypair: anchor.web3.Keypair;
}> => {
  const storeKeypair = anchor.web3.Keypair.generate();
  const storeAdminKeypair = payer.payer;

  const name = "Store Name";
  const description = "Store Description";

  try {
    const tx = await nftShopProgram.methods
      .createStore(name, description)
      .accounts({
        store: storeKeypair.publicKey,
        storeAdmin: storeAdminKeypair.publicKey,
      })
      .signers([storeKeypair, storeAdminKeypair])
      .rpc();
    console.log("Transaction [Create Store]", tx);
  } catch (error) {
    console.log(error);
  }

  return {
    storeKeypair,
    storeAdminKeypair,
  };
};

export const initSellingResource = async ({
  provider,
  nftShopProgram,
  nftMinterProgram,
  payer,
  storeKeypair,
  storeAdminKeypair,
}: {
  provider: anchor.Provider;
  nftShopProgram: anchor.Program<NftShop>;
  nftMinterProgram: anchor.Program<NftMinter>;
  payer: anchor.Wallet;
  storeKeypair: anchor.web3.Keypair;
  storeAdminKeypair: anchor.web3.Keypair;
}): Promise<{
  sellingResourceKeypair: anchor.web3.Keypair;
  sellingResourceOwnerKeypair: anchor.web3.Keypair;
  vaultKeypair: anchor.web3.Keypair;
}> => {
  const sellingResourceKeypair = anchor.web3.Keypair.generate();
  const sellingResourceOwnerKeypair = await createSystemAccount({
    provider,
    payer,
  });

  const resourceMintKeypair = anchor.web3.Keypair.generate();

  const name = "Solana Course NFT";
  const symbol = "SOLC";
  const uri =
    "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json";
  const creators = [
    {
      address: sellingResourceOwnerKeypair.publicKey,
      share: 100,
      verified: false,
    },
  ];
  const sellerFeeBasisPoints = 100;
  const is_mutable = true;

  const [metadata] = findMetadataAddress({
    mint: resourceMintKeypair.publicKey,
  });

  // Create Token
  try {
    const tx = await nftMinterProgram.methods
      .createToken(
        name,
        symbol,
        uri,
        creators,
        sellerFeeBasisPoints,
        is_mutable
      )
      .accounts({
        payer: payer.publicKey,
        mintAccount: resourceMintKeypair.publicKey,
        mintAuthority: payer.publicKey,
        updateAuthority: sellingResourceOwnerKeypair.publicKey,
        metadataAccount: metadata,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([resourceMintKeypair, payer.payer, sellingResourceOwnerKeypair])
      .rpc();
    console.log("Transaction [Create Token]", tx);
  } catch (error) {
    console.log(error);
  }

  const [masterEdition, masterEditionBump] = findEditionAddress({
    mint: resourceMintKeypair.publicKey,
  });

  const maxSupply = 1;

  const resourceToken = anchor.utils.token.associatedAddress({
    mint: resourceMintKeypair.publicKey,
    owner: payer.publicKey,
  });

  // Mint Token
  try {
    const tx = await nftMinterProgram.methods
      .mintToken(new BN(maxSupply))
      .accounts({
        payer: payer.publicKey,
        mintAccount: resourceMintKeypair.publicKey,
        mintAuthority: payer.publicKey,
        updateAuthority: sellingResourceOwnerKeypair.publicKey,
        associatedTokenAccount: resourceToken,
        metadataAccount: metadata,
        editionAccount: masterEdition,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([resourceMintKeypair, payer.payer, sellingResourceOwnerKeypair])
      .rpc();
    console.log("Transaction [Mint Token]", tx);
  } catch (error) {
    console.log(error);
  }

  const [vaultOwner, vaultOwnerBump] = findVaultOwnerAddress(
    resourceMintKeypair.publicKey,
    storeKeypair.publicKey
  );

  const vaultKeypair = anchor.web3.Keypair.generate();
  await createTokenAccount({
    provider,
    payer,
    tokenAccount: vaultKeypair,
    mint: resourceMintKeypair.publicKey,
    owner: vaultOwner,
  });

  // Init Selling Resource
  try {
    const tx = await nftShopProgram.methods
      .initSellingResource(masterEditionBump, vaultOwnerBump, new BN(maxSupply))
      .accounts({
        store: storeKeypair.publicKey,
        storeAdmin: storeAdminKeypair.publicKey,
        sellingResource: sellingResourceKeypair.publicKey,
        sellingResourceOwner: sellingResourceOwnerKeypair.publicKey,
        metadata,
        masterEdition,
        resourceMint: resourceMintKeypair.publicKey,
        resourceToken: resourceToken,
        vault: vaultKeypair.publicKey,
        vaultOwner,
      })
      .signers([storeAdminKeypair, sellingResourceKeypair])
      .rpc();
    console.log("Transaction [Init Selling Resource]", tx);
  } catch (error) {
    console.log(error);
  }

  return {
    sellingResourceKeypair,
    sellingResourceOwnerKeypair,
    vaultKeypair,
  };
};

describe("nft_shop", async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const nftShopProgram = anchor.workspace.NftShop as Program<NftShop>;
  const nftMinterProgram = anchor.workspace.NftMinter as Program<NftMinter>;

  const payer = provider.wallet as anchor.Wallet;

  const NFT_PRICE = 10;

  it("Test User Flow", async () => {
    const { storeKeypair, storeAdminKeypair } = await createStore({
      nftShopProgram,
      payer,
    });

    const {
      sellingResourceKeypair,
      sellingResourceOwnerKeypair,
      vaultKeypair,
    } = await initSellingResource({
      provider,
      nftShopProgram,
      nftMinterProgram,
      payer,
      storeKeypair,
      storeAdminKeypair,
    });

    const marketKeypair = anchor.web3.Keypair.generate();

    const treasuryMintKeypair = anchor.web3.Keypair.generate();
    await createMintAccount({
      provider,
      payer,
      mint: treasuryMintKeypair,
    });

    const [treasuryOwner, treasuryOwnerBump] = findTreasuryOwnerAddress(
      treasuryMintKeypair.publicKey,
      sellingResourceKeypair.publicKey
    );

    const treasuryHolderKeypair = anchor.web3.Keypair.generate();
    await createTokenAccount({
      provider,
      payer,
      tokenAccount: treasuryHolderKeypair,
      mint: treasuryMintKeypair.publicKey,
      owner: treasuryOwner,
    });

    const marketName = "Market Name";
    const marketDescription = "Market Description";
    const mutable = true;
    const price = new BN(NFT_PRICE);
    const piecesInOneWallet = new BN(1);
    const startDate = new BN(Math.round(Date.now() / 1000));
    const endDate = null;

    // Create Market
    try {
      const tx = await nftShopProgram.methods
        .createMarket(
          treasuryOwnerBump,
          marketName,
          marketDescription,
          mutable,
          price,
          piecesInOneWallet,
          startDate,
          endDate
        )
        .accounts({
          market: marketKeypair.publicKey,
          store: storeKeypair.publicKey,
          sellingResource: sellingResourceKeypair.publicKey,
          sellingResourceOwner: sellingResourceOwnerKeypair.publicKey,
          treasuryMint: treasuryMintKeypair.publicKey,
          treasuryHolder: treasuryHolderKeypair.publicKey,
          treasuryOwner,
        })
        .signers([marketKeypair, sellingResourceOwnerKeypair])
        .rpc();
      console.log("Transaction [Create Market]", tx);
    } catch (error) {
      console.log(error);
    }

    const newMarketName = "New Market Name";
    const newMarketDescription = "New Market Description";
    const newMutable = null;
    const newPrice = null;
    const newPiecesInOneWallet = null;

    // Change Market
    try {
      const tx = await nftShopProgram.methods
        .changeMarket(
          newMarketName,
          newMarketDescription,
          newMutable,
          newPrice,
          newPiecesInOneWallet
        )
        .accounts({
          market: marketKeypair.publicKey,
          sellingResourceOwner: sellingResourceOwnerKeypair.publicKey,
        })
        .signers([sellingResourceOwnerKeypair])
        .rpc();
      console.log("Transaction [Change Market]", tx);
    } catch (error) {
      console.log(error);
    }

    const sellingResourceData =
      await nftShopProgram.account.sellingResource.fetch(
        sellingResourceKeypair.publicKey
      );

    const [tradeHistory, tradeHistoryBump] = findTradeHistoryAddress(
      payer.publicKey,
      marketKeypair.publicKey
    );

    const [vaultOwner, vaultOwnerBump] = findVaultOwnerAddress(
      sellingResourceData.resource,
      sellingResourceData.store
    );

    const userTokenAccount = anchor.web3.Keypair.generate();
    await createTokenAccount({
      provider,
      payer,
      tokenAccount: userTokenAccount,
      mint: treasuryMintKeypair.publicKey,
    });
    await mintTo({
      provider,
      mint: treasuryMintKeypair.publicKey,
      destination: userTokenAccount.publicKey,
      authority: payer.payer,
      amount: NFT_PRICE,
    });

    const newMintKeypair = anchor.web3.Keypair.generate();
    await createMintAccount({
      provider,
      payer,
      mint: newMintKeypair,
    });

    const newTokenAccountMint = anchor.web3.Keypair.generate();
    await createTokenAccount({
      provider,
      payer,
      tokenAccount: newTokenAccountMint,
      mint: newMintKeypair.publicKey,
    });
    await mintTo({
      provider,
      mint: newMintKeypair.publicKey,
      destination: newTokenAccountMint.publicKey,
      authority: payer.payer,
      amount: 1,
    });

    const [metadata] = findMetadataAddress({
      mint: sellingResourceData.resource,
    });
    const [masterEdition] = findEditionAddress({
      mint: sellingResourceData.resource,
    });

    const [editionMarker] = findEditionMarkerAddress({
      mint: sellingResourceData.resource,
      supply: sellingResourceData.supply.toNumber(),
    });
    const [newMetadata] = findMetadataAddress({
      mint: newMintKeypair.publicKey,
    });
    const [newEdition] = findEditionAddress({
      mint: newMintKeypair.publicKey,
    });

    const [primaryMetadataCreators, primaryMetadataCreatorsBump] =
      findPrimaryMetadataCreatorsAddress(metadata);

    const primaryRoyaltiesHolder = anchor.web3.Keypair.generate();
    const creators = [
      {
        address: primaryRoyaltiesHolder.publicKey,
        share: 100,
        verified: false,
      },
    ];

    // Save Primary Metadata Creators
    try {
      const tx = await nftShopProgram.methods
        .savePrimaryMetadataCreators(primaryMetadataCreatorsBump, creators)
        .accounts({
          metadataUpdateAuthority: sellingResourceOwnerKeypair.publicKey,
          metadata,
          primaryMetadataCreators,
        })
        .signers([sellingResourceOwnerKeypair])
        .rpc();
      console.log("Transaction [Save Primary Metadata Creators]", tx);
    } catch (error) {
      console.log(error);
    }

    const userWalletKeypair = payer.payer;

    // Buy
    try {
      const tx = await nftShopProgram.methods
        .buy(tradeHistoryBump, vaultOwnerBump)
        .accounts({
          market: marketKeypair.publicKey,
          sellingResource: sellingResourceKeypair.publicKey,
          userTokenAccount: userTokenAccount.publicKey,
          userWallet: userWalletKeypair.publicKey,
          tradeHistory,
          treasuryHolder: treasuryHolderKeypair.publicKey,
          newMetadata,
          newEdition,
          masterEdition,
          newMint: newMintKeypair.publicKey,
          editionMarker,
          vault: sellingResourceData.vault,
          vaultOwner,
          newTokenAccount: newTokenAccountMint.publicKey,
          metadata,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([userWalletKeypair])
        .rpc();
      console.log("Transaction [Buy]", tx);
    } catch (error) {
      console.log(error);
    }

    // Close Market
    try {
      const tx = await nftShopProgram.methods
        .closeMarket()
        .accounts({
          market: marketKeypair.publicKey,
          sellingResourceOwner: sellingResourceOwnerKeypair.publicKey,
        })
        .signers([sellingResourceOwnerKeypair])
        .rpc();
      console.log("Transaction [Close Market]", tx);
    } catch (error) {
      console.log(error);
    }

    const [payoutTicket, payoutTicketBump] = findPayoutTicketAddress(
      marketKeypair.publicKey,
      primaryRoyaltiesHolder.publicKey
    );

    const destination = anchor.utils.token.associatedAddress({
      mint: treasuryMintKeypair.publicKey,
      owner: primaryRoyaltiesHolder.publicKey,
    });

    const primaryMetadataCreatorsData: anchor.web3.AccountMeta[] = [];
    for (const creator of [primaryMetadataCreators]) {
      primaryMetadataCreatorsData.push({
        pubkey: creator!,
        isWritable: true,
        isSigner: false,
      });
    }

    const payerKeypair = payer.payer;

    // Withdraw
    try {
      const tx = await nftShopProgram.methods
        .withdraw(treasuryOwnerBump, payoutTicketBump)
        .accounts({
          market: marketKeypair.publicKey,
          sellingResource: sellingResourceKeypair.publicKey,
          metadata,
          treasuryHolder: treasuryHolderKeypair.publicKey,
          treasuryMint: treasuryMintKeypair.publicKey,
          treasuryOwner,
          destination,
          funder: primaryRoyaltiesHolder.publicKey,
          payer: payerKeypair.publicKey,
          payoutTicket,
        })
        .remainingAccounts(primaryMetadataCreatorsData)
        .signers([payerKeypair])
        .rpc();
      console.log("Transaction [Withdraw]", tx);
    } catch (error) {
      console.log(error);
    }

    const claimTokenAccount = anchor.web3.Keypair.generate();
    await createTokenAccount({
      provider,
      payer,
      tokenAccount: claimTokenAccount,
      mint: sellingResourceData.resource,
    });

    // Claim Resource
    try {
      const tx = await nftShopProgram.methods
        .claimResource(vaultOwnerBump)
        .accounts({
          market: marketKeypair.publicKey,
          treasuryHolder: treasuryHolderKeypair.publicKey,
          sellingResource: sellingResourceKeypair.publicKey,
          sellingResourceOwner: sellingResourceOwnerKeypair.publicKey,
          vault: vaultKeypair.publicKey,
          metadata,
          vaultOwner,
          destination: claimTokenAccount.publicKey,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([sellingResourceOwnerKeypair])
        .rpc();
      console.log("Transaction [Claim Resource]", tx);
    } catch (error) {
      console.log(error);
    }
  });
});
