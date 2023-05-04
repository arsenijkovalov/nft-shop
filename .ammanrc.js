module.exports = {
  validator: {
    // By default Amman will pull the account data from the accountsCluster (can be overridden on a per account basis)
    accountsCluster: 'https://api.metaplex.solana.com',
    accounts: [
        {
          label: 'Token Metadata Program',
          accountId:'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
          // marking executable as true will cause Amman to pull the executable data account as well automatically
          executable: true,
        }
      ]
  }
}
