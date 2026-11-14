# Security model: status

Newest first.

## 2026-11-14 · Services row partly in place

- PostgreSQL is reachable only through a Unix socket in a user-only directory. It opens no network listener.
- The run directory is created user-only, and one that belongs to another user is refused.
- Nothing else in the table exists yet.
