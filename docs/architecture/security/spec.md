# Security model, stated honestly

Spec section 10.

| Subject | Guarantee | Mechanism |
| --- | --- | --- |
| First-party apps | Least-privilege data access; no secrets; no network. Trusted code, not sandboxed | Per-app tokens checked by the broker; build rules forbid direct network, file and process use in app crates; CI enforces the dependency allow-list |
| Third-party apps (Views) | Confined by construction | They are data rendered by the window; logic runs as limited WebAssembly |
| Agents | Bounded authority | One Cell per agent, no secrets inside, one wire out |
| Services | Not reachable by other local users or apps | Unix sockets with user-only permissions, per-service secrets |
| Remote model calls | Never by default; always logged | Model gateway and the global toggle |

Not claimed: protection from malware already running as the user; confinement of first-party app code beyond the rules above; anything about a signed, notarized build until signing lands.
