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

  it("Create Burry Escrow Test", async () => {
    await safeAirdrop(user.publicKey, provider.connection)

    console.log("Escrow Account: ", escrowState.toBase58())

    // Send transaction
    const amtInLamps = new anchor.BN(10)
    const unlockPrice = new anchor.BN(15)

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

      console.log("On-chain unlock price:", newAccount.unlockPrice.toNumber())
      console.log("Amount in escrow:", await provider.connection.getBalance(escrowState, "confirmed"))

      // Check whether the data on-chain is equal to local 'data'
      assert(unlockPrice.eq(newAccount.unlockPrice))
    } catch (e) {
      console.log(e)
    }
  })

  it("Withdraw from Burry Escrow", async () => {
    const switchboardProgram = await SwitchboardProgram.load(
      "devnet",
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      user
    )

    const aggregatorAccount = new AggregatorAccount(switchboardProgram, solUsedSwitchboardFeed)

    try {
      const tx = await program.methods.withdraw({ maxConfidenceInterval: null })
      .accounts({
        user: user.publicKey,
        escrowAccount: escrowState,
        feedAggregator: aggregatorAccount.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
    })
      .signers([user])
      .rpc()

      console.log("Your transaction signature", tx)

    } catch (e) {
      console.log(e)
    }
  })
})
