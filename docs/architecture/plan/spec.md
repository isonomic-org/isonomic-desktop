# Milestones, risks and open decisions

Spec sections 13 to 15.

## Milestones

| # | Ships | Gate |
| --- | --- | --- |
| M0 Skeleton | Engine supervising bundled PostgreSQL; broker with the system interface; window with the rail and a Home tab showing live service status; menu bar icon; universal unsigned app | Force-kill the database: it recovers and the window shows the transition. Quit stops everything in order. The report prints cold-start time |
| M1 Stack | Remaining services, lazy tier, profiles applied through config, RAM gate wired, logs view, start at login | All core services healthy in under 10 seconds on the Intel test machine; login leaves the engine up with no window |
| M2 Models and Chat | Model gateway, local provider, model library, Models settings, the Chat app as the first tab | Local-only provably never leaves the machine; the toggle cuts a remote stream mid-flight |
| M3 Keys and backups | Vault, master key, passkey wrap, recovery kit, archive, Backups settings | Wipe and restore onto a second machine from a synced folder |
| M4 App model | Manifests, Store tab, per-app tokens and schemas, second app (Docs) | A second app cannot read the first app's data |
| M5 Connections | Accounts, MCP, egress gateway, Activity tab | Two accounts of one provider; per-tool rules enforced |
| M6 Agents | Cell supervisor, Agents app | Per the Cells record |

Signing, updates and the Store's on-chain checks follow M6. No milestone starts until the previous gate passes.

## The Milestone 0 gate, on Omar's Mac

1. Launching the app shows the window with the rail and the database reported healthy.
2. Force-killing the database process makes it restart on its own, and the window shows degraded then healthy without being touched.
3. Quit stops everything in order and leaves no process behind.
4. The report prints cold-start time and bundle size.

## Risks and spikes

| Risk | Spike or mitigation |
| --- | --- |
| Menu bar icon and resident process under GPUI | First week. Fallback: a tiny helper process owns the icon |
| GPUI on the Intel test machine | First window build runs there before any UI work continues |
| Native passkeys with the key-wrapping extension | Before M3. Fallback: recovery kit mandatory, passkey wrap added later |
| GPUI and the component library are both pre-1.0 | One re-export crate; upgrades on a schedule |
| Bundle size | Trim the database tools; measure in the M0 report |
| Cells need newer macOS and Apple Silicon | Intel machines run agents as functions only. Minimum platform is an open decision |
| Scope | One gate at a time |

## Open decisions

1. Minimum macOS version, and whether Intel Macs get Cells at all.
2. Keep or replace the object-storage and cache services after M3.
3. Keep running in the background as the default on Quit.
4. Which chain SDK replaces the earlier one for account operations.
5. License for the window and the engine.
