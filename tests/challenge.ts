import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Keypair, PublicKey } from "@solana/web3.js";
import { BN, Program } from "@coral-xyz/anchor";
import { Stryd } from "../target/types/stryd";

const IDL = require("../target/idl/stryd.json");

const challenge_address = new PublicKey("ADPCeyuUkasdBcnGRDoFR4ZzmGKbsjtLW9KJwMpdX5Ce");

describe("Stryd", () => {
    it("Create Challenge", async () => {
        const context = await startAnchor("",[{name: "stryd", programId: challenge_address}],[]);
        const provider = new BankrunProvider(context);

        const challengeProgram = new Program<Stryd>(IDL, provider);

        await challengeProgram.methods.createChallenge(
            new BN(1),
            new BN(10),
            "Challenge 1"

        ).accounts({
            creator: provider.wallet.publicKey,
        }).signers([
            provider.wallet.payer,
        ]).rpc();

        const [challenge] = PublicKey.findProgramAddressSync([ Buffer.from("challenge"), provider.wallet.publicKey.toBuffer(), new BN(1).toArrayLike(Buffer, "le", 8)], challenge_address);
        const challengeAccount = await challengeProgram.account.challenge.fetch(challenge);



        console.log(challengeAccount);

        const challengeId = new BN(1);

        const [challengePda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("challenge"),
              creator.toBuffer(), 
              challengeId.toArrayLike(Buffer, "le", 8),
            ],
            program.programId
          );

        await challengeProgram.methods.joinChallenge(
            challengeId
        ).accounts({
            joiner: provider.wallet.publicKey,
            creator: provider.wallet.publicKey,
        }).signers([
            provider.wallet.payer,
        ]).rpc();

        console.log(challengeAccount);
    })

})
