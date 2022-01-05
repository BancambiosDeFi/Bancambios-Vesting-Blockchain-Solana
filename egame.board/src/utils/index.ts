import { LAMPORTS_PER_SOL } from "@solana/web3.js";
const bigNumber = require("bignumber.js");

const converterBN = (number: { toString: () => string; }): string => (new bigNumber(number.toString())
    .dividedBy(LAMPORTS_PER_SOL)
    .toString()
);

export default converterBN;