import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { BurryOracleProgram } from "../target/types/burry_oracle_program"
import { AggregatorAccount, SwitchboardProgram, SwitchboardTestContext } from '@switchboard-xyz/solana.js'
import { OracleJob } from '@switchboard-xyz/common'
import { NodeOracle } from "@switchboard-xyz/oracle"
import assert from "assert"
import Big from 'big.js'
import { safeAirdrop } from './utils/utils'
import { userKeypair1, solUsedSwitchboardFeed, usdc_usdFeed } from './TestKeypair/testKeypair'

describe("burry-oracle-program", async () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const program = anchor.workspace.BurryOracleProgram as Program<BurryOracleProgram>
  let switchboard: SwitchboardTestContext
  let oracle: NodeOracle
  let aggregatorAccount: AggregatorAccount
  let switchboardProgram: SwitchboardProgram
  let user = userKeypair1
  let user2 = new anchor.web3.Keypair()

  before(async () => {
    switchboard = await SwitchboardTestContext.loadFromProvider(provider, {
      name: "Test Queue",
      // You can provide a keypair to so the PDA schemes dont change between test runs. Will create one if one does not already exist.
      keypair: SwitchboardTestContext.loadKeypair("./TestKeypair/queue_keypair.json"),
      queueSize: 10,
      reward: 0,
      minStake: 0,
      oracleTimeout: 900,
      // aggregators will not require PERMIT_ORACLE_QUEUE_USAGE before joining a queue
      unpermissionedFeeds: true,
      unpermissionedVrf: true,
      enableBufferRelayers: true,
      oracle: {
        name: "Test Oracle",
        enable: true,
        stakingWalletKeypair: SwitchboardTestContext.loadKeypair(
          "./TestKeypair/staking_wallet_keypair.json"
        ),
      },
    })

    oracle = await NodeOracle.fromReleaseChannel({
      chain: "solana",
      // use the latest testnet (devnet) version of the oracle
      releaseChannel: "testnet",
      // disables production capabilities like monitoring and alerts
      network: "localnet",
      rpcUrl: provider.connection.rpcEndpoint,
      oracleKey: switchboard.oracle.publicKey.toBase58(),
      // path to the payer keypair so the oracle can pay for txns
      secretPath: switchboard.walletPath || "~/.config/solana/id.json",
      // set to true to suppress oracle logs in the console
      silent: true,
      // optional env variables to speed up the workflow
      envVariables: {
        VERBOSE: "1",
        DEBUG: "1",
        DISABLE_NONCE_QUEUE: "1",
        DISABLE_METRICS: "1",
      },
    })

    // start the oracle and wait for it to start heartbeating on-chain
    await oracle.startAndAwait()
  })

  after(() => {
    oracle?.stop()
  })

  it("Create Burry Escrow", async () => {
    // fetch switchboard devnet program object
    switchboardProgram = await SwitchboardProgram.load(
      "devnet",
      new anchor.web3.Connection("https://api.devnet.solana.com"),
      user
    )
    aggregatorAccount = new AggregatorAccount(switchboardProgram, solUsedSwitchboardFeed)

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

  it("Attempt to withdraw with incorrect Feed account", async () => {
    // derive escrow address
    const [user2EscrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user2.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )

    try {
      const tx = await program.methods.withdraw({ maxConfidenceInterval: null })
      .accounts({
        user: user2.publicKey,
        escrowAccount: user2EscrowState,
        feedAggregator: usdc_usdFeed,
        systemProgram: anchor.web3.SystemProgram.programId
    })
      .signers([user2])
      .rpc()
    } catch (e) {
      console.log(e.error.errorMessage)
      assert(e.error.errorMessage == 'An address constraint was violated')
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

  /*
  it("Pass in closed feed account and receive escrow funds", async () => {
    // create static feed account
    const [staticFeedAccount, staticFeedAccountState] = await switchboard.createStaticFeed({
      value: 10
    })

    // create new escrow with static feed account
    const [escrowState] = await anchor.web3.PublicKey.findProgramAddressSync(
      [user.publicKey.toBuffer(), Buffer.from("MICHAEL BURRY")],
      program.programId
    )
    
    const failUnlockPrice = new anchor.BN(20)
    const amtInLamps = new anchor.BN(100)

    // create escrow transaction
    try {
      console.log("Depositing funds into escrow...")
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
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }

    try {
      console.log("Attempting to withdraw BEFORE feed account has been closed...")
      const tx = await program.methods.withdrawClosedFeedFunds()
      .accounts({
        user: user.publicKey,
        escrowAccount: escrowState,
        closedFeedAccount: staticFeedAccount.publicKey,
      })
      .signers([user])
      .rpc()
      await provider.connection.confirmTransaction(tx, "confirmed")

    } catch (e) {
      console.log(e.error.errorMessage)
      assert(e.error.errorMessage == 'Feed account is not closed, must be closed to redeem with the withdraw_closed_feed_funds instruction.')
    }

    // close feed account
    console.log("Closing feed account...")
    await staticFeedAccount.close()

    let initialUserBalance = await provider.connection.getBalance(user.publicKey, "confirmed")

    // redeem escrow after feed account has been closed
    try {
      console.log("Attempting to withdraw after feed account has been closed...")
      const tx = await program.methods.withdrawClosedFeedFunds()
      .accounts({
        user: user.publicKey,
        escrowAccount: escrowState,
        closedFeedAccount: staticFeedAccount.publicKey,
      })
      .signers([user])
      .rpc()

      await provider.connection.confirmTransaction(tx, "confirmed")
      console.log("Your transaction signature", tx)

      let currentUserBalance = await provider.connection.getBalance(user.publicKey, "confirmed")
      let closedFeed = await provider.connection.getAccountInfo(staticFeedAccount.publicKey)
      console.log("Feed info:", closedFeed)

      console.log("Initial user balance: ", initialUserBalance)
      console.log("Current user balance: ", currentUserBalance)
      assert(currentUserBalance > initialUserBalance)
    } catch (e) {
      console.log(e)
      assert.fail(e)
    }

  })
  */
})