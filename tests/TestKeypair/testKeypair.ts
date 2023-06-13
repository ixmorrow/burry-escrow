import * as anchor from '@coral-xyz/anchor'

export const userKeypair1 = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(
    [
        66, 100,  68,   3,  40, 140, 169,  29, 162,  89, 109,
        224, 199,  68, 213,  65, 167, 125, 109, 204,  88, 161,
        179,  87, 154, 106,  67, 129, 230,  91,   9,   3, 194,
        232, 243,  12, 178, 239, 241,  78, 220, 232, 115, 113,
        107,  67,  74,  85,  82,  61, 112,   1, 147, 209,  13,
        228,  46,   0, 185,  95, 222,  72,  18,   9
    ]
))

export const switchboardDevnetProgramID = new anchor.web3.PublicKey("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f")
export const solUsedSwitchboardFeed = new anchor.web3.PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR")
export const usdc_usdFeed = new anchor.web3.PublicKey("BjUgj6YCnFBZ49wF54ddBVA9qu8TeqkFtkbqmZcee8uW")