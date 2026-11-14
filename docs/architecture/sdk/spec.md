# The contract: isonomic-sdk

Spec section 5.

```
crates/
  proto      versioned request, response and event types
  client     async client over the socket, reconnect, subscriptions
  app        the trait a first-party app implements (needs isonomic-ui)
  testkit    an in-memory Capsule for tests
```

- **Wire format v0.** Length-prefixed JSON messages: request, response, event. Chosen so a person can read the traffic and paste it into a bug report. A binary encoding can replace it behind the same types.
- **Streams.** Subscriptions deliver events for service status, logs, live queries, document updates and model tokens.
- **Versioning.** The first message negotiates a protocol version. The engine supports the current and previous version.
- **Interfaces, in the order they are built:** system (status, logs, profile, shutdown), models, sql, content, vault (shell only), archive (shell only), events, connections, egress, vector, graph, functions, identity, cells.
- **Long term.** The same interfaces are described in WIT so WebAssembly functions and Views import exactly what native apps call.

## The app trait

A first-party app is a crate that exposes:

- its manifest: identity, icon, capabilities, schemas, model and connection needs
- a constructor that receives a Capsule client scoped to its token and returns its root view
- optional hooks: open a deep link, handle a search query, respond to unload

The window links the app crates and registers them at build time. Installing an app from the Store enables it, shows its manifest, creates its schemas and issues its token.

## Milestone 0 scope

`proto` (types for `system`), `client` (reconnect and subscriptions) and `testkit` (an in-memory engine). The `app` crate is not built.
