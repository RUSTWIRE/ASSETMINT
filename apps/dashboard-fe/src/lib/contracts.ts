// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Deployed SilverScript covenant contracts on Kaspa Testnet-12.
// These are the P2SH addresses and deployment TX hashes for each contract.

export const DEPLOYED_CONTRACTS = {
  clawback: {
    name: "Clawback",
    scriptSize: 161,
    txId: "6080b47733e42d1cff8597cab14b2a412d8e423bed36add64d980c158f5c77eb",
    p2shAddress:
      "kaspatest:ppztfhpzpxkqkxum37ymje2dehrj0l49t3c75a3k4pu3jzp37edn202ftet8y",
    entrypoints: ["ownerSpend", "issuerClawback"],
  },
  rwaCore: {
    name: "RwaCore",
    scriptSize: 395,
    txId: "d7ed495882132765eb1c1dabd2cb378e3dbe5f39b1770c0313e54782e5a6baec",
    p2shAddress:
      "kaspatest:prhl2h3vdsq32u8068dqtm6x3qazz4nz9jkv9lq2u4j80c8c0ldrwqt9z3d2t",
    entrypoints: ["zkTransfer", "adminUpdate"],
  },
  stateVerity: {
    name: "StateVerity",
    scriptSize: 316,
    txId: "94c50753b05e7d998af30fa51aad4d27f2e7fdd0e9ae48b655255b94d129fe5f",
    p2shAddress:
      "kaspatest:pq6xyf8f4tzpeuz4s6yy8063j6g6dwv0a4lcerv4uc98m99shgpcsftdcl5d7",
    entrypoints: ["updateState", "managerReclaim"],
  },
  zkKycVerifier: {
    name: "ZkKycVerifier",
    scriptSize: 396,
    txId: "c29499adf3d1353ce914d8e61184357c31d479039ee91c41a09345953bf93c45",
    p2shAddress:
      "kaspatest:pzhqgz42uftlpg2hpekn7sh48ddmmny9wrql8nczk2nuevsjgp7cz99szuyqs",
    entrypoints: ["verifyProof", "updateVerifierKey"],
  },
  reserves: {
    name: "Reserves",
    scriptSize: 372,
    txId: "346fdbd30cf88fd6e1ba60444cb3ea892cf59bc807019106b7e6f8f18f012e1b",
    p2shAddress:
      "kaspatest:prlsah5judppj9np80zzp4qyrf90ccjnvd3u9uvhx8gzf7pjej33vkl0ln4vg",
    entrypoints: ["withdraw", "deposit", "custodianReclaim"],
  },
  htlc: {
    name: "HTLC",
    scriptSize: 195,
    txId: "1347b397ff482c8ed1f8b914eab5102425c891111c38016008b98df6d3390528",
    p2shAddress:
      "kaspatest:prrz0mrxc3020lajzm4zj9gtf9q0nwp7ku05sen9fz4rlldw4d9z5t2ftdll5",
    entrypoints: ["claimWithPreimage", "refundAfterTimeout"],
  },
  dividend: {
    name: "Dividend",
    scriptSize: 406,
    txId: "6ec163e1882bda2ac238626112e525d20d90c1bb569828f1fd279e7aea294c9c",
    p2shAddress:
      "kaspatest:prrf9w05fgvpq8k40t24pdcst0r99504fq50uma0q233a2fh8kln2gxllvp6p",
    entrypoints: ["claimDividend", "issuerTopUp", "issuerReclaim"],
  },
} as const;

export type ContractName = keyof typeof DEPLOYED_CONTRACTS;
