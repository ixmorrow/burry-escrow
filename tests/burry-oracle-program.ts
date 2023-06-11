import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { BurryOracleProgram } from "../target/types/burry_oracle_program"
import { AggregatorAccount, SwitchboardProgram } from '@switchboard-xyz/solana.js'
import assert from "assert"
import Big from 'big.js'
import { safeAirdrop } from './utils/utils'
import { userKeypair1, solUsedSwitchboardFeed, switchboardDevnetProgramID } from './TestKeypair/testKeypair'

describe("burry-oracle-program", async () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.BurryOracleProgram as Program<BurryOracleProgram>
  const provider = anchor.AnchorProvider.env()

  let user = userKeypair1
  let user2 = new anchor.web3.Keypair()

  it("Create Burry Escrow", async () => {
    // fetch switchboard devnet program object
    const switchboardProgram = await SwitchboardProgram.load(
      "devnet",
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      user
    )
    const aggregatorAccount = new AggregatorAccount(switchboardProgram, solUsedSwitchboardFeed)

    // derive escrow state account
    const [escrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )
    console.log("Escrow Account: ", escrowState.toBase58())

    // airdrop to user
    await safeAirdrop(user.publicKey, provider.connection)

    // fetch latest SOL price
    const solPrice: Big | null = await aggregatorAccount.fetchLatestValue()
    if (solPrice === null) {
      throw new Error('Aggregator holds no value')
    }
    const failUnlockPrice = new anchor.BN(solPrice.plus(10).toNumber())
    const amtInLamps = new anchor.BN(100)

    // Send transaction
    try {
      const tx = await program.methods.deposit(amtInLamps, failUnlockPrice)
      .accounts({
        user: user.publicKey,
        escrowAccount: escrowState,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([user])
      .rpc()

      await provider.connection.confirmTransaction(tx, "confirmed")
      console.log("Your transaction signature", tx)

      // Fetch the created account
      const newAccount = await program.account.escrowState.fetch(
        escrowState
      )

      let escrowBalance = await provider.connection.getBalance(escrowState, "confirmed")
      console.log("On-chain unlock price:", newAccount.unlockPrice.toNumber())
      console.log("Amount in escrow:", escrowBalance)

      // Check whether the data on-chain is equal to local 'data'
      assert(failUnlockPrice.eq(newAccount.unlockPrice))
      assert(escrowBalance > 0)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }
  })

  it("Attempt to withdraw while price is below UnlockPrice", async () => {
    // derive escrow address
    const [escrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )
    
    // send tx
    try {
      const tx = await program.methods.withdraw({ maxConfidenceInterval: null })
      .accounts({
        user: user.publicKey,
        escrowAccount: escrowState,
        feedAggregator: solUsedSwitchboardFeed,
        systemProgram: anchor.web3.SystemProgram.programId
    })
      .signers([user])
      .rpc()

      await provider.connection.confirmTransaction(tx, "confirmed")
      console.log("Your transaction signature", tx)

    } catch (e) {
      // verify tx returns expected error
      console.log(e.error.errorMessage)
      assert(e.error.errorMessage == 'Current SOL price is not above Escrow unlock price.')
    }
  })

  it("Create New Burry Escrow with new UnlockPrice", async () => {
    // fetch switchboard devnet program object
    const switchboardProgram = await SwitchboardProgram.load(
      "devnet",
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      user
    )
    const aggregatorAccount = new AggregatorAccount(switchboardProgram, solUsedSwitchboardFeed)
    
    // derive escrow address
    const [user2EscrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user2.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )
    console.log("Escrow Account: ", user2EscrowState.toBase58())

    // airdrop to user2
    await safeAirdrop(user2.publicKey, provider.connection)

    // fetch latest SOL price
    const solPrice: Big | null = await aggregatorAccount.fetchLatestValue()
    if (solPrice === null) {
      throw new Error('Aggregator holds no value')
    }
    const successUnlockPrice = new anchor.BN(solPrice.minus(10).toNumber())
    const amtInLamps = new anchor.BN(100)

    // send tx
    try {
      const tx = await program.methods.deposit(amtInLamps, successUnlockPrice)
      .accounts({
        user: user2.publicKey,
        escrowAccount: user2EscrowState,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([user2])
      .rpc()

      await provider.connection.confirmTransaction(tx, "confirmed")
      console.log("Your transaction signature", tx)

      // Fetch the created account
      const newAccount = await program.account.escrowState.fetch(
        user2EscrowState
      )

      let escrowBalance = await provider.connection.getBalance(user2EscrowState, "confirmed")
      console.log("On-chain unlock price:", newAccount.unlockPrice.toNumber())
      console.log("Amount in escrow:", escrowBalance)

      // Check whether the data on-chain is equal to local 'data'
      assert(successUnlockPrice.eq(newAccount.unlockPrice))
      assert(escrowBalance > 0)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }
  })

  it("Withdraw from Burry Escrow", async () => {
    // derive escrow address
    const [user2EscrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user2.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )

    // send tx
    try {      
      let initialUserBalance = await provider.connection.getBalance(user2.publicKey, "confirmed")
      const tx = await program.methods.withdraw({ maxConfidenceInterval: null })
      .accounts({
        user: user2.publicKey,
        escrowAccount: user2EscrowState,
        feedAggregator: solUsedSwitchboardFeed,
        systemProgram: anchor.web3.SystemProgram.programId
    })
      .signers([user2])
      .rpc()

      await provider.connection.confirmTransaction(tx, "confirmed")
      console.log("Your transaction signature", tx)


      let currentUserBalance = await provider.connection.getBalance(user2.publicKey, "confirmed")
      console.log("Initial user balance: ", initialUserBalance)
      console.log("Current user balance: ", currentUserBalance)

      assert(currentUserBalance > initialUserBalance)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }
  })
})