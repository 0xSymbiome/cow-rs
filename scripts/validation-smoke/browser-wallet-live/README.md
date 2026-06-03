# Browser Wallet Live Canary

This runbook covers the manual browser-extension canary for `cow-sdk-browser-wallet`.
It intentionally stays outside the deterministic crate test lanes because it
depends on an installed wallet extension, unlocked account state, chain
inventory, and provider approval prompts. The canonical
`examples/wasm/cow-trader-dioxus` example is the vehicle: it exercises the
public browser-wallet contract (discover, connect, sign, switch chain, trade)
end to end.

## Prerequisites

- Chromium with the target wallet extension installed from the vendor source.
- A funded throwaway wallet account on Sepolia (a little test ETH or COW).
- No production mainnet signing account loaded in the same browser profile.
- Local checkout built from the commit under review.
- The Dioxus CLI installed (`cargo install dioxus-cli --locked`, provides `dx`).

## Procedure

1. Build and serve the example:

   ```bash
   cd examples/wasm/cow-trader-dioxus
   dx serve --platform web
   ```

2. Open the printed URL (Dioxus serves on `http://localhost:8080` by default) in
   the prepared browser profile.
3. Click **Discover wallets** and confirm exactly the intended injected provider
   appears; with several extensions installed, none is auto-selected.
4. Click **Connect** on the intended wallet, approve only the expected origin,
   and confirm the session line reports the expected account and chain.
5. Click **Sign message (personal_sign)** and confirm a signature returns.
6. With the wallet on another network, run **Sign & submit swap** and confirm the
   example asks the wallet to switch to Sepolia (`wallet_switchEthereumChain`)
   and only signs after the refreshed session reports Sepolia.
7. Disconnect, reconnect, and repeat the account and chain checks.

## Pass Criteria

- Provider discovery is explicit and deterministic; no wallet is auto-selected.
- Chain switching reports success only after the wallet session reflects the
  requested chain.
- `personal_sign` returns a signature for the displayed payload.
- Disconnect and reconnect do not leak the prior session state.
- No unexpected browser devtools errors appear outside wallet-extension UI noise.

## Failure Handling

Record the wallet name, wallet version, browser version, chain id, example
commit, observed step, and the exact browser or wallet error. Treat chain
mismatch, silent provider auto-selection, stale accounts after disconnect, or
signing a payload different from the displayed one as release-blocking until
triaged.
