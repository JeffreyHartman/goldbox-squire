# Issue tracker: local markdown

Issues and specs for this repo live as markdown files in `.scratch/`. The
repo has a GitHub remote, and it deliberately has no GitHub Issues: the program
is being built in public, and a working notebook of half-formed tickets is not
what the issue list is for. This can change later. Until it does, `.scratch/`
is the tracker.

`.scratch/` is committed. Jeff works from two machines, so the map, the tickets
and the research have to travel with the code.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at
  `.scratch/<feature-slug>/issues/<NNN>-<slug>.md`, never a single combined
  tickets file
- **Ticket numbers are three digits**, `001` upward, continuing the existing
  run. This repo deviates from the two-digit default because the numbers are
  cited in commit messages and across the map, and renumbering would break
  those citations.
- Triage state is recorded as a `Status:` line near the top of each issue file
  (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a
  `## Comments` heading

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if
needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or
the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` (the Notes / Decisions-so-far / Fog body).
- **Child ticket**: `.scratch/<effort>/issues/NNN-<slug>.md`, three digits, with
  the question in the body. A `Type:` line records the ticket type
  (`research`/`prototype`/`grilling`/`task`); a `Status:` line records
  `claimed`/`resolved`.
- **Research write-ups**: `.scratch/<effort>/research/NNN-<slug>.md`, one per
  research ticket, linked from the ticket's answer.
- **Prototypes**: `.scratch/<effort>/<name>/`, whatever shape the question
  needs, with a `README.md` saying what to look at and what to decide. A
  prototype ticket is answered by something Jeff can open, not by prose.
- **Blocking**: a `Blocked by: NNN, NNN` line near the top. A ticket is
  unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open,
  unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set
  `Status: resolved`, then append a context pointer (gist + link) to the map's
  Decisions-so-far in `map.md`.

## Efforts that exist

- `.scratch/squire-v1/` — the v1 map, 32 tickets, and its research. v1 is
  built. The map's destination is now the feature roadmap that follows it, and
  the roadmap itself lives at `docs/roadmap.md`, not in a ticket.
- `.scratch/squire-hud/` — the HUD map and tickets 033 upward. Ticket numbers
  continue the run rather than restarting, because a commit message citing
  `033` must name exactly one ticket across the whole repo.

## Not the tracker

- `docs/adr/` holds decision records, not tickets. A ticket asks; an ADR
  records what was decided. See `domain.md`.
- `docs/roadmap.md` is a standing document, edited for as long as the program
  is worked on. It never closes, so it is not a ticket.
