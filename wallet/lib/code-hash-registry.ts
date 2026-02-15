// Maps blake3 hash of contract WASM → app type ID.
// These hashes are computed from the prebuilt WASM binaries in examples/*/target/.
// If a contract is recompiled, these hashes will change and need updating.
export const CODE_HASH_REGISTRY: Record<string, string> = {
  "59f94cd20c7d313a110069d6090f47105ad1fd7f8dfd0ceb8ba1cc7371622f7a": "crowdfund",
  "7e882ccd4946d07ad6cfb2a46d03de7b2b5e7bff87f843038afbdf00fff59c3f": "escrow",
  "2e86152b2c038aa8ca5c3f8396ce4e2d9c5dc20c0531a4bc9f3aa75c2b8c2e41": "governance",
  "9331efd7bc8378427a5a0277a529115fc1a89dcef2e68c8c23afdd9d3e13500a": "treasury",
  "e67881d58c9a1a93237a7da31f8f1e3f507855ae1d86cc58f1c6a95f41c3c16e": "vesting",
  "3f23bbcf5f17dca806c3d464b72f5aac203389e2b72c7076112c5b03a9a72ec6": "launchpad",
  "f7438fea067a569e826aa1354c07ca1c5f7550fabb1ac30e9156f38bddcf42c5": "splitter",
  "8a04857937c118b15a4c054672ebc4811a17fe40034d8c0647d9e57068af6bd1": "staking",
  "02e3a671cbc3dac425be8fe4dab2eeb54ad5d719a8b8a5c67fbba48b3900f031": "swap",
  "5a5a0f81f85f4108f72865962e92d2d3022121a54097462f31e415b2d8c1cd12": "airdrop",
  "a22882287038dd7ff2e69f952a90d4ff8a438fdc3720d7e44312c0bab0af4688": "timelock",
};

export function getAppTypeForCodeHash(hash: string): string | undefined {
  return CODE_HASH_REGISTRY[hash.toLowerCase()];
}

export function getCodeHashForAppType(appType: string): string | undefined {
  return Object.entries(CODE_HASH_REGISTRY).find(([, t]) => t === appType)?.[0];
}
