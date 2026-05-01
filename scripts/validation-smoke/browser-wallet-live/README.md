# Browser Wallet Live Canary

This runbook covers the manual browser-extension canary for `cow-sdk-browser-wallet`.
It intentionally stays outside the deterministic Playwright lane because it depends
on an installed wallet extension, unlocked account state, chain inventory, and
provider approval prompts.

## Prerequisites

- Chromium with the target wallet extension installed from the vendor source.
- A funded throwaway wallet account on the selected test chain.
- No production mainnet signing account loaded in the same browser profile.
- Local checkout built from the commit under review.

## Procedure

1. Start the browser-wallet console:

   ```bash
   bun run --cwd e2e/browser-wallet serve:console
   ```

2. Open `http://127.0.0.1:4174` in the prepared browser profile.
3. Confirm the page discovers exactly the intended wallet provider.
4. Connect the wallet and approve only the expected origin.
5. Switch to the intended test chain through the console control and confirm the wallet UI reports the same chain.
6. Request accounts, request chain id, and sign the deterministic typed-data fixture shown by the console.
7. Disconnect, reconnect, and repeat the account and chain-id checks.

## Pass Criteria

- Provider discovery is explicit and deterministic.
- Chain switching reports success only after the wallet session reflects the requested chain.
- Typed-data signing returns a signature for the displayed deterministic payload.
- Disconnect and reconnect do not leak the prior session state.
- No unexpected browser console errors appear outside wallet-extension UI noise.

## Failure Handling

Record the wallet name, wallet version, browser version, chain id, console commit,
observed step, and the exact browser or wallet error. Treat chain mismatch,
silent provider auto-selection, stale accounts after disconnect, or signing a
payload different from the displayed fixture as release-blocking until triaged.
