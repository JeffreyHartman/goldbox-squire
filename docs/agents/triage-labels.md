# Triage labels

The skills speak in terms of five canonical triage roles. This file maps those
roles to the strings used in this repo's issue tracker.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the
corresponding label string from this table.

## Where a label goes

There is no label API, because the tracker is markdown files. A label is written
as a `Triage:` line near the top of the issue file, next to `Type:` and
`Status:`.

## Triage state is not wayfinder state

A ticket carries both, and they answer different questions.

- `Status:` is wayfinder's lifecycle: `open`, `claimed`, `resolved`, `done`.
  Whether the work has been picked up and finished.
- `Triage:` is one of the five roles above. Whether the ticket is ready, and for
  whom.

Existing v1 tickets carry `Status:` and `Type:` only. They predate this file.
Do not backfill `Triage:` across them; add it when a ticket is actually triaged.
