# Broker and capability tokens

Spec section 4.6.

- Listens on a Unix socket in the user's Capsule directory with user-only permissions. The path is kept short to fit the OS limit.
- The window authenticates as the shell and receives a session. Each first-party app tab gets its own capability token derived from its approved manifest, so data access is least-privilege per app even inside one process.
- Every call is checked against the token and recorded for the Activity tab.

## Milestone 0 scope

The socket server with the `system` interface only:

- status of every service
- subscribe to status changes
- recent logs
- hardware profile
- ordered shutdown
