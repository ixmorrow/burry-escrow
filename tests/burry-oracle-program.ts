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

  const [escrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
    [user.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
    program.programId
  )

  // const switchboardProgram = await SwitchboardProgram.load(
  //   "devnet",
  //   new anchor.web3.Connection("https://api.devnet.solana.com"),
  //   user
  // )
  // const aggregatorAccount = new AggregatorAccount(switchboardProgram, solUsedSwitchboardFeed)

  // const solPrice: Big | null = await aggregatorAccount.fetchLatestValue()
  // if (solPrice === null) {
  //   throw new Error('Aggregator holds no value')
  // }

  it("Create Burry Escrow", async () => {
    await safeAirdrop(user.publicKey, provider.connection)

    console.log("Escrow Account: ", escrowState.toBase58())

    // Send transaction
    const amtInLamps = new anchor.BN(100)
    // const failUnlockPrice = new anchor.BN(solPrice.plus(10).toNumber())
    // const successUnlockPrice = new anchor.BN(solPrice.minus(10).toNumber())
    const unlockPrice = new anchor.BN(10)

    try {
      const tx = await program.methods.deposit(amtInLamps, unlockPrice)
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
      assert(unlockPrice.eq(newAccount.unlockPrice))
      assert(escrowBalance > 0)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }
  })

  it("Withdraw from Burry Escrow", async () => {

    try {
      let initialUserBalance = await provider.connection.getBalance(user.publicKey, "confirmed")
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


      let currentUserBalance = await provider.connection.getBalance(user.publicKey, "confirmed")
      console.log("Initial user balance: ", initialUserBalance)
      console.log("Current user balance: ", currentUserBalance)

      assert(currentUserBalance > initialUserBalance)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }
  })
})
