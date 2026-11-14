# Data, identity and payments, backups

Spec sections 7 to 9. None of this is in Milestone 0.

## Data

- **Records** live in PostgreSQL: rows, typed fields, transactions, per-row history. Changes reach the window as live-query events.
- **Documents** are collaborative data structures (pages, boards, graphs, views) stored by the engine, with full history, versions and offline edits.
- **On disk**, everything sits under one Capsule directory in the user's application-support folder: `services/<name>/`, `documents/`, `blobs/`, `models/`, `logs/`, `registry.db`, `run/` for sockets. Models are excluded from backups and re-downloaded by hash.

## Identity and payments

- **Account.** A passkey creates a smart account with no seed phrase. The account address is the workspace root. No custodied keys, ever.
- **Native path.** Passkey ceremonies use platform APIs; account operations are built in Rust.
- **Trust boundary.** Signing happens in the window process through the platform's passkey sheet. The engine never holds signing keys; it consumes a short-lived signed session to check entitlements and authorize key operations.
- **Payments.** One stable-dollar asset only. App licenses and subscriptions are on-chain tokens checked by the engine. Card checkout opens in the system browser and returns by deep link.
- **Two recoveries.** Guardians can restore an account. Only a second wrap of the master key (another passkey or the recovery kit) can restore data. The UI never blurs the two.

## Backups

- Destinations are folders. The OS's own sync client moves the files to a cloud drive or anything else.
- A snapshot is a consistent checkpoint across services: a database dump, object and document stores, the vault (still encrypted), manifests and settings.
- Stored deduplicated and encrypted in large immutable files, index written last. A daily snapshot costs only the day's changes.
- Default daily, plus snapshot now in the menu bar. Keeps 7 daily, 4 weekly, 12 monthly.
- A weekly integrity check and a monthly test restore into a scratch directory. The status tile turns amber when the last good snapshot is older than twice the interval.
- Restore covers a whole Capsule on a fresh install, one app, or one document at a point in time.
