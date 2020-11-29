import {
  PublicKey,
  solana, Wallet, BPFLoader, SPLToken,
  ProgramAccount,
  Deployer,
} from "solray"

import { promises as fs } from "fs"

import { Faucet } from "./faucet"

async function main() {
  const master = {
    mnemonic: 'control bulk thing never memory turn toss mass just youth ginger spot',
    seed: '65ec16a63ef5cbc0d1e5604a4ac04671c6c55f6dccfe841263ea6ec2b1af147d79636f65336aea58e71d03f620a00d579601b11256e5735235ee4e2b1ca4b68a'
  }

  const conn = solana.connect("dev")
  const wallet = await Wallet.fromMnemonic(master.mnemonic, conn)

  console.log("using wallet", wallet.address)


  const deployer = await Deployer.open("deploy.json")
  const spltoken = new SPLToken(wallet)

  const faucetProgram = await deployer.ensure("faucetProgram", async () => {
    const bpfLoader = new BPFLoader(wallet)

    const soPath = "build/program.so"
    const soData = await fs.readFile(soPath)
    const program = await bpfLoader.load(soData)

    return program
  })

  const facuetPubkey = faucetProgram.publicKey

  const testToken = await deployer.ensure("testToken", async () => {
    const token = await spltoken.initializeMint({
      mintAuthority: wallet.account.publicKey,
      decimals: 8,
    })

    return token
  })

  // The trick to create a program token account is to create a normal token account
  // (we can throw away the secret key), and set the token account's owner to the
  // program account address (which the program can sign with a seed).
  //
  // See: https://docs.solana.com/implemented-proposals/program-derived-addresses
  // Also See: https://docs.solana.com/implemented-proposals/cross-program-invocation


  // NOTE: because of an implementation quirk of how program account is generated,
  // we choose only 32 bytes of the public key to use as seed.
  const faucetTokenOwner = await ProgramAccount.forSeed(Buffer.from(testToken.publicKey.toBuffer()).slice(0, 30), facuetPubkey)
  console.log("faucet token owner", {
    address: faucetTokenOwner.address,
    seed: faucetTokenOwner.noncedSeed.toString("hex"),
    nonce: faucetTokenOwner.nonce,
  })

  // mint tokens to faucet token account
  const faucetTokenAccount = await deployer.ensure("faucetTokenAccount", async () => {
    const faucetTokenAccount = await spltoken.initializeAccount({
      token: testToken.publicKey,
      owner: faucetTokenOwner.pubkey
    })

    await spltoken.mintTo({
      token: testToken.publicKey,
      to: faucetTokenAccount.publicKey,
      amount: BigInt(10000e8),
      authority: wallet.account,
    })

    return faucetTokenAccount
  })

  const receiverTokenAccount = await deployer.ensure("receiverTokenAccount", async () => {
    return spltoken.initializeAccount({
      token: testToken.publicKey,
      owner: wallet.pubkey,
    })
  })

  const receiverAddress = receiverTokenAccount.publicKey.toString()

  const receiverTokenAccountInfo = await spltoken.accountInfo(receiverTokenAccount.publicKey)
  console.log("receiver token account", {
    address: receiverAddress,
    owner: receiverTokenAccountInfo.owner.toString(),
    mint: receiverTokenAccountInfo.mint.toString(),
  })

  const faucet = new Faucet(wallet, facuetPubkey)
  await faucet.request({
    receiver: receiverTokenAccount.publicKey,
    tokenAccount: faucetTokenAccount.publicKey,
    tokenOwner: faucetTokenOwner,
  })

  console.log(`sent one token from faucet to: ${receiverAddress}`)
  console.log(`view on devnet: https://explorer.solana.com/address/${receiverAddress}?cluster=devnet`)
}

main().catch(err => console.log({ err }))
