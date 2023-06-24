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

export const queueKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(
    [
        224,  71, 221, 100, 247,  28, 176,   3,  76, 160,  38,
        204, 112,  11, 150, 100, 239, 207, 110, 141, 228,  13,
        112,  14, 186,  81, 217, 126,  70,  49,  90, 123,  11,
        142,  33, 175,  96, 191,  35, 190, 171,  97,  48, 209,
        159, 232, 227, 182, 163, 204, 160, 178,  50, 191,  87,
        169, 243,  38, 223, 158,  63, 164, 151, 250
    ]
))

export const stakingWalletKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(
    [
        14, 188,  86,  31, 250,   2,   3, 214,  66, 231, 212,
        166, 204, 174,  27, 215, 186, 159,  72, 105,  83, 108,
        195,  47, 171,  73, 181, 190,  86, 148, 109,  24,  44,
        232,  33,  94, 217, 157, 111,  32,  93, 129, 243, 239,
        177, 224,  13, 191, 159, 167,  95, 251, 125,  69,  10,
        159, 196,  58, 165, 185,  32,  34, 176, 179
    ]
))

export const jobAccount = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(
    [
        71, 205, 202,  42, 235, 125,  52, 136, 190,  99,  37,
        227,  19,  23,  32,  73, 240, 123, 125, 102, 115,  87,
        139,  62,  16,  77, 175, 191, 227, 102,  45, 106, 255,
        187, 199,  11, 218, 107, 144, 129, 202, 236,  33, 193,
        222,  25, 167,  96, 202, 125, 245, 137,  69, 151, 178,
        245, 147, 214,  86,  48, 164, 220, 250, 110
    ]
))

export const switchboardDevnetProgramID = new anchor.web3.PublicKey("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f")
export const solUsedSwitchboardFeed = new anchor.web3.PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR")
export const usdc_usdFeed = new anchor.web3.PublicKey("BjUgj6YCnFBZ49wF54ddBVA9qu8TeqkFtkbqmZcee8uW")