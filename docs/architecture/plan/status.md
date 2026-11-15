# Milestones and gate: status

Newest first.

## 2026-11-15 · Milestone 1 gate: the report passes; login checked by hand, partly

`cargo xtask report` now runs the Milestone 1 gate (desktop #8): every core service healthy, each one force-killed and back, ordered stop with nothing left (pid files included), then a warm start on the same Capsule against the plan's ten seconds. The lazy item is n/a: no lazy service ships in Milestone 1. Start at login is printed as steps to do by hand.

The block from the Intel Mac (`--host-only`, the engine on capsule #23's branch):

```
report      Isonomic Desktop, Milestone 1 · 2026-11-15 07:03 UTC
machine     MacBookPro15,1 · Intel(R) Core(TM) i7-8850H CPU @ 2.60GHz · macOS 15.7.9 · x86_64 · 16 GB · profile standard
toolchain   rustc 1.98.1 (48a229cea 2026-09-01)
window      0.0.1 · isonomic-desktop 71ac659 (this branch)
engine      0.0.1 · isonomic-capsule 3570bca
sdk         isonomic-sdk 6ce97ba · SDK_REF window 6962099 · engine 6962099
ui          isonomic-ui 15f2f4c · UI_REF 15f2f4c
postgresql  postgres (PostgreSQL) 18.4
app         ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app · x86_64 · 59.6 MB · ad-hoc signed
              MacOS/isonomic-desktop (x86_64)                        24.5 MB
              MacOS/isonomic-capsule (x86_64)                         4.0 MB
              Resources/sidecars/postgres-x86_64-apple-darwin        27.2 MB
              Resources/sidecars/valkey                               3.8 MB
engine bin  ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/MacOS/isonomic-capsule
engine log  hardware profile: standard (detected, 16.0 GiB)
            binary postgres: ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/Resources/sidecars/postgres-x86_64-apple-darwin/bin/postgres (executable)
            binary initdb: ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/Resources/sidecars/postgres-x86_64-apple-darwin/bin/initdb (executable)
            binary cache: ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/Resources/sidecars/valkey/valkey-server (executable)
            postgres: first-time setup done
            cold start: 175 ms
cold start  175 ms (engine: core services up) · healthy 2.0 s after launch
gate
  1 every core service healthy             PASS  postgres and cache healthy 2.0 s after the first launch (first-time setup included)
  2 force-killed postgres comes back       PASS  killed pid 9448 → healthy again in 1.4 s as pid 9467 · restarts 1 · 0 of 8 old workers left
  2 force-killed cache comes back          PASS  killed pid 9449 → healthy again in 1.1 s as pid 9534 · restarts 1 · 0 of 0 old workers left
  3 the lazy service starts on use         n/a   no lazy service ships in Milestone 1 (the vector database waits for wave 3); the engine's tests cover the lazy tier
  4 ordered stop, nothing left running     PASS  stopped in 0.2 s: every service, then the engine; socket and pid files removed; 0 of 11 processes left
  5 warm start under 10 s                  PASS  every core service healthy 0.1 s after launch (target under 10 s); stopped clean again
  6 cold start and bundle size reported    PASS  cold start above; app 59.6 MB
  by hand: start at login
    1. open the app once from dist/ (installs the agent; Start at Login is checked in the menu bar)
    2. log out and back in: no window opens; open the app: one engine, all services healthy
    3. kill -9 the engine: the window shows it reconnecting, then healthy
    4. Quit from the menu: nothing runs afterwards and nothing restarts it
result      PASS (start at login: by hand, above)
```

**Start at login, by hand** (detail in `window/status.md`): installed on first launch, the engine runs under launchd, Quit leaves it stopped, a simulated login brings every service up with no window in under 2 s, and a killed engine is back and healthy in 5.1 s. Still to do by Omar: a real logout and login, and the menu bar switch.

**Gate:** the report's items pass on the Intel Mac. The login half passes as far as it can be driven from here.

## 2026-11-14 (night) · Gate item 4 and the universal app

The Milestone 0 gate passes on Omar's Intel Mac (sysctl: MacBookPro15,1, i7-8850H, 16 GB, macOS 15.7.9).

| Gate item | Result |
| --- | --- |
| 1. Launching shows the window with the rail and the database healthy | **Pass.** Report: healthy. Window: checked by eye in the dev bundle (evening entry below), and the universal `Isonomic.app` launched from `dist/` shows the rail and "All services healthy" |
| 2. Force-killing the database: it restarts and the window shows the transition | **Pass.** Report: back in 1.4 to 3.0 s, 0 of 8 old workers left. Window: checked by eye in the dev bundle |
| 3. Quit stops everything in order | **Pass.** Report: stopped in 0.1 to 0.2 s, clean PostgreSQL shutdown, 0 processes left. Window: Quit checked by eye in the dev bundle |
| 4. The report prints cold-start time and bundle size | **Pass.** `cargo xtask report`; universal app 106.0 MB |

| Run (Intel Mac) | Cold start (engine) | Healthy after launch | Recovery after kill | Stop |
| --- | --- | --- | --- | --- |
| Host-only app, first run of new binaries | 626 ms | 11.8 s | 1.9 s | 0.1 s |
| Universal app, first run of new binaries | 313 ms | 7.1 s | 1.5 s | 0.1 s |
| Universal app, quiet machine | 75 ms | 1.8 s | 1.4 s | 0.1 s |
| Universal app, another session running a release build | 1186 ms | 54.1 s | 3.0 s | 0.2 s |

"Healthy after launch" includes first-time database setup, which every run does because each report uses a fresh scratch Capsule. The first run of new binaries also waits for macOS's one-off check of programs it has not seen. Every timing here is on the Intel Mac; Apple Silicon is covered by CI only.

Still to look at once, by a person: the menu bar light's colors and the status item's own Open and Quit, in the universal app.

```text
------------------------------------------------------------------------
report      Isonomic Desktop, Milestone 0 · 2026-11-15 00:25 UTC
machine     MacBookPro15,1 · Intel(R) Core(TM) i7-8850H CPU @ 2.60GHz · macOS 15.7.9 · x86_64 · 16 GB · profile standard
toolchain   rustc 1.98.1 (48a229cea 2026-09-01)
window      0.0.1 · isonomic-desktop d2b43ac
engine      0.0.1 · isonomic-capsule 2257305
sdk         isonomic-sdk 6ce97ba · SDK_REF window 6962099 · engine 6962099
postgresql  postgres (PostgreSQL) 18.4
app         ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app · universal (x86_64, aarch64) · 106.0 MB · ad-hoc signed
              MacOS/isonomic-desktop (x86_64 arm64)                  42.4 MB
              MacOS/isonomic-capsule (x86_64 arm64)                   7.8 MB
              Resources/sidecars/postgres-x86_64-apple-darwin        27.2 MB
              Resources/sidecars/postgres-aarch64-apple-darwin       28.3 MB
engine bin  ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/MacOS/isonomic-capsule
engine log  hardware profile: standard
            binary postgres: ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/Resources/sidecars/postgres-x86_64-apple-darwin/bin/postgres (executable)
            binary initdb: ~/Code/isonomic/repos/isonomic-desktop/dist/Isonomic.app/Contents/Resources/sidecars/postgres-x86_64-apple-darwin/bin/initdb (executable)
            postgres: first-time setup done
            cold start: 1186 ms
cold start  1186 ms (engine: core services up) · healthy 54.1 s after launch
gate
  1 engine and PostgreSQL healthy          PASS  healthy 54.1 s after launch
  2 force-killed PostgreSQL comes back     PASS  killed pid 67232 → healthy again in 3.0 s as pid 67255 · restarts 1 · 0 of 8 old workers left
  3 ordered stop, nothing left running     PASS  stopped in 0.2 s: PostgreSQL shut down cleanly, then the engine; socket removed; 0 of 10 processes left
  4 cold start and bundle size reported    PASS  cold start above; app 106.0 MB
  window: the rail, the light through kill and recovery, and Quit are checked by eye
result      PASS
------------------------------------------------------------------------
```

## 2026-11-14 (evening) · Gate items 1 to 3 pass on the Intel test Mac

Run on the 2020 Intel MacBook Pro (macOS 15.7), with the window in a dev app bundle (`Isonomic Dev.app`, ad-hoc signed) holding the window, the engine and PostgreSQL 18.4.

| Gate item | Result |
| --- | --- |
| 1. Launching shows the window with the rail and the database healthy | **Pass.** The window started the engine itself; it showed the rail and "All services healthy" with postgres green |
| 2. Force-killing the database: it restarts and the window shows degraded then healthy, untouched | **Pass.** After `kill -9` the window showed "Services starting" with postgres Restarting (amber), then "All services healthy" with "1 restart", without being touched. The engine reports a crash as starting, not degraded; degraded is the circuit breaker |
| 3. Quit stops everything in order, nothing left behind | **Pass.** Quit from the app menu: window, engine and PostgreSQL all gone within 1 s; PostgreSQL logged a clean shutdown and removed its lock file; the engine socket was removed |
| 4. The report prints cold-start time and bundle size | **Not yet.** Cold start is logged (about 0.2 s since the probe ramp). No report command and no universal app yet |

Also verified: closing the window hides the app while the engine and database keep running; launching the app again shows the window; the menu bar status item is on screen.

Not verified: the status light's colors as drawn, and clicking the menu bar item's own Open and Quit. No background tool can see or click a status item; a person has to look once.

Still to build: `cargo xtask report` and the universal unsigned app.

## 2026-11-14 (later still) · Both halves exist; nothing has run on a Mac

Engine side of items 1 to 3 proven on Linux; window side proven in headless window tests. Cold start was 5.0 s, all of it probe cadence.

## 2026-11-14 · Milestone 0 in progress; gate not attempted
