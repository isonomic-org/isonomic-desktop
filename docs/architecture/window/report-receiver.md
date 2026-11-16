# Problem reports: the receiver's contract

What a service must do to receive the problem reports Isonomic sends. Either a small web service or a database function behind an HTTPS endpoint can implement it. The window's side is `crates/reports/src/send.rs`.

## The request

One `POST` per report, to one URL that the build is configured with.

| Part | Value |
| --- | --- |
| Method | `POST` |
| URL | The configured URL, used exactly as given. Must be `https://` (plain `http://` is sent only to this Mac's loopback, for tests) |
| `Content-Type` | `application/json` |
| `Idempotency-Key` | The report's id: 32 lowercase hex characters. The same as the document's `id` |
| `Authorization` | `Bearer <key>`, only when the build is configured with a key |
| Body | One JSON document, format 1, UTF-8, at most 65,536 bytes |

Nothing else is sent: no cookies, no query string the URL doesn't already have, no other identifying headers.

## The document

The body is exactly what the person saw under Show report. It is the report as it was kept on the Mac, already scrubbed, with two fields added at the top level:

- `id`: the report's id, as in `Idempotency-Key`.
- `isonomic_id`: the person's Isonomic ID. Present only when the person ticked "Include my Isonomic ID so you can contact me about this report" for this report. A report sent automatically never has it.

The other top-level fields come from format 1, the SDK's `isonomic-reports`, described in its `docs/reports.md`:

| Field | What |
| --- | --- |
| `format` | `1`. A receiver must accept unknown fields, and should refuse a `format` it doesn't know with 422 |
| `kind` | `panic`, `hang`, `app_failed`, `app_error`, `service_failed`, `model_server_failed` |
| `component` | `{ id, version? }`: `window`, an app's id, `engine`, `engine/<service>`, `engine/<runtime>` |
| `product` | `{ version, build, arch, pins: { sdk, chat, engine } }` |
| `system` | `{ macos, profile, memory_gb, capsule_format }` |
| `when` | `{ unix_secs, uptime_secs }` |
| `message`, `backtrace`, `log_tail`, `tab` | Scrubbed text |
| `service`, `model` | The engine's two failure kinds only |

## The id, and repeats

The window sends a report again when an attempt failed, and the same report may come from a later launch. It always carries the same `id`.

A receiver must treat a second request with an `id` it has already stored as success: answer 200, and store nothing new.

- **A report made since format 1 carried an `id`:** its own, random, made when the report was written.
- **One kept from before that:** the first 128 bits of the SHA-256 of the Capsule's id and the report's file name.

Either way an id is stable for one report, and two reports from one Mac share nothing a receiver could link.

## Answers

| Status | Meaning | What the window does |
| --- | --- | --- |
| 201 | Stored | Deletes its copy |
| 200 | Already stored (a repeat `id`) | Deletes its copy |
| Any other 2xx | Taken | Deletes its copy |
| 400 | Not JSON, or not a format 1 report | Keeps it; never sends it again |
| 401, 403 | Missing or wrong key | Keeps it; never sends it again |
| 413 | Over the size limit | Keeps it; never sends it again |
| 422 | A format the receiver doesn't know | Keeps it; never sends it again |
| Any other 4xx | Refused | Keeps it; never sends it again |
| 5xx | The receiver couldn't take it now | Tries again, as below |

To rate-limit, answer 503, not 429. A 4xx is never tried again.

The response body is ignored, so keep it empty or small.

## Retries

After a network error, a timeout (30 s per attempt) or a 5xx, the window tries again. It waits 30 s, then doubles the wait each time up to an hour, and gives up a day after the first attempt.

A report that wasn't sent stays on the Mac. It is asked about at the next launch under Ask each time, or sent at the next look under Send automatically. Kept reports are deleted after 30 days, or beyond the 20 newest.

## What a receiver must not do

- Log or forward the `Authorization` header.
- Ask for, or keep, anything to tie a report to a person beyond `isonomic_id`, which is there only because they chose it.
