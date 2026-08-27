# Wayfinder map: the Goldbox Squire HUD

Label: `wayfinder:map`

## Destination

A HUD: Squire's live party on screen beside the game, in a window the user
places and sizes, at whatever size that turns out to be.

v1 reprints a table into the terminal it was launched from. That works and it
is honest, but it is not something you glance at mid-fight. The HUD is.

## Notes

**Where this came from.** A throwaway spike, written by DeepSeek v4 Pro on a
worktree at `/home/jeff/git/goldbox-squire-temp`, built a ratatui interface on
top of `c21f75b`: a gold double rule, a five-row block-letter wordmark, a
sixteen-column tab list down the left, and the party table. Five tabs, four of
them placeholders. The palette and the number-key idea were good. The layout
was not: it costs sixteen columns permanently to advertise four screens that do
not exist, and it assumes a terminal large enough to hold a wordmark.

The spike is not merged and is not a base. `theme.rs` and the number-key jump
come across. `ui.rs` does not.

**Wayland forbids a client positioning its own window.** No flag, no protocol,
no workaround. This is why GBC's pin-above-DOSBox trick cannot be ported, and
it is already recorded in `docs/roadmap.md`. What Squire *can* do is set its
own app-id, so the user writes one KWin or Hyprland rule once and the window
lands where they want it forever. Size we can set. Position we cannot, ever.

**No number from one person's monitors belongs in the code.** An early draft of
this design measured Jeff's ultrawide and picked breakpoints from it. That was
wrong: the repo is public and the program is meant to be shared. The layout
answers "does this fit", never "is this the size I expect".

**Skills each session must consult.** `codebase-design` when the layout plan's
interface is designed. `prototype` for the mockups. `tdd` at the layout plan
seam, which is agreed and recorded below.

## Decisions so far

Settled in the grilling session of 2026-08-25, before any code.

- **The word is HUD.** The concept, not the technology. TUI means the terminal
  implementation of the HUD. A GUI version, if it ever exists, is another
  implementation of the same thing. "Dock" is not a word this project uses.
- **One layout, driven by rules, not named modes.** No `Wide`, `Tall` or
  `Fullscreen` in the code. Rules answer "does this fit" and add or drop parts
  accordingly. The names survive only in documentation, describing regions of a
  continuous space. Named layouts were rejected because a monitor landing
  between two names gets the worse one.
- **The seam is a pure function.** Rows, columns and a party go in; a
  description of what is shown comes out. Tested with no terminal. The drawing
  code takes that description and stays thin. This is the seam tests are
  written at, agreed before the work started.
- **Content differs by size, it does not merely reflow.** Small is the party
  and nothing else. Roomy is where a map or a journal will live, if they ever
  do. Pretending one content model reflows into every size gives every panel a
  cramped mode nobody wants to read.
- **The unit is a card, not a table row.** Settled after the first mockups.
  One card per character, and the layout is how six cards are arranged: one
  across, two, three, or six, by a rule that asks how wide a card wants to be.
  A single table stretched to every width was the same table everywhere, which
  is what the sizes were supposed to stop. GBC's own overlay is six cards.
- **A character can carry several conditions at once.** The card holds a list,
  one line each, above the ability scores. Squire reads one status byte today,
  so the list has one item; the effects read fills it later without changing
  the card.
- **Drop order, last to go first**: name, current and maximum hit points,
  conditions, class, level, armor class. Conditions rank high because a silent
  `poisoned` is the thing you most want to notice without looking away from the
  game. Ability scores are not in this order at all: they are off by default
  and a key turns them on, because they barely change during play and six
  numbers on every card make the party look busy.
- **A rule and a preference are different questions.** The layout plan answers
  "does this fit". A key answers "did the user ask for it". The plan takes the
  toggles as inputs and never decides one, or a preference ends up buried
  inside a fitting rule where nobody can find it.
- **The sidecar starts at 60 columns.** The width Squire opens at before it has
  remembered anything. Wide enough for one column of six cards with the class
  on the name line.
- **No left menu, and no wordmark except when roomy.** Navigation is number
  keys with the panel name on the status line, which costs no rows and no
  columns and is the Gold Box idiom already. A HUD that reminds you what it is
  called is a HUD that does not respect the rows.
- **Stale numbers dim.** Ticket 007 settled that Squire never shows stale
  numbers as live. A reprinting table could satisfy that by not printing. A
  persistent screen has to say it, so the party block dims and the status line
  says why. Dimming is unmissable in peripheral vision, which is where a HUD is
  read from.
- **Geometry is remembered, globally.** Recorded when the HUD exits, reused
  next launch. Not per game: window size is a property of where the user sits,
  not of which game they loaded.
- **The HUD becomes `gbs`, and `--plain` is the escape.** `args.rs` already
  argues that an argument required to make the program work is not an argument,
  which is why there is no `--watch`. The same reasoning applies here. Pipes
  and scripts get `--plain`.
- **Squire will spawn its own window, but not first.** The launching terminal
  keeps the emulator handle and becomes the log: rescans, anchor losses,
  emulator output. It never drops back to a prompt. Deferred deliberately: the
  spawn is only small once the HUD already survives any size, and doing it
  first would let one blessed size be designed for.
- **The terminal table is data the user can extend.** Compiled-in defaults for
  the terminals that handle app-id and cell sizing cleanly, merged under a user
  file keyed by terminal name. An unknown terminal is still spawned, with a
  message saying its size cannot be set. Adding a terminal in five years is a
  file, not a rebuild and not a pull request.
- **A GUI map stays open, as a third crate.** If the map ever needs pixels,
  `squire-gui` sits beside `squire-cli` in the workspace and depends on
  `squire-core`. Same solution, separate project, shared library. Not decided,
  not blocked, and not a reason to abandon the terminal: a character-grid map
  might read fine.

- **The vocabulary is written down.** HUD, TUI, plain output, layout plan, drop
  order, roomy, wordmark and app id are defined in `CONTEXT.md`, and the
  roadmap's HUD section no longer promises three named layouts. See
  `issues/033-hud-vocabulary-and-roadmap.md`.

- **The watch loop has a drawing seam.** The loop moved into the library as
  `watch::watch` and draws through a `Screen` trait, with the printed table as
  the first implementation. The pause and the keyboard are a second trait,
  `Keys`, because they are one wait. See
  `issues/035-prefactor-renderer-seam.md`.

- **The terminal table exists.** foot, alacritty and kitty are compiled in with
  three fields each: how to name the window, how to ask for a size in cells,
  and what goes before the command. A user file in Squire's config folder is
  merged over it by name. See `issues/042-terminal-table.md`.

- **The layout plan is a pure function, and it is written.** Rows, columns, a
  party and the toggles go in; a `Plan` comes out, saying what each card holds,
  where the cards go, whether the wordmark is drawn and whether the party
  dims. 034's seven recorded sizes are pinned as tests. See
  `issues/036-layout-plan.md`.

- **The HUD is what `gbs` does, and `--plain` is the escape.** The party is
  drawn on a screen you glance at, it reflows as the window changes, the
  terminal comes back after an error or a panic, and a pipe still gets the
  table. See `issues/037-hud-draws-and-reflows.md`.

- **Stale numbers dim, and waiting is not the same as lost.** The plan's
  `Liveness` splits a run that never found a party from one that lost the
  anchor, because the session reports both as not-found. Only the second dims.
  See `issues/038-stale-numbers-dim.md`.

- **The window size is remembered under `[hud]`.** Read from the terminal at
  every draw, written on the way out, ignored when it is nonsense. Nothing
  acts on it until 048. See `issues/039-remember-window-size.md`.

- **The keys are settled, and the slot repick survives raw mode.** `q` quits,
  the arrows move the highlight, `a` and `c` are the toggles, the number keys
  are reserved, and Enter steps the HUD aside so the one wizard asks the one
  question. See `issues/040-hud-keys.md`.

- **Naming the window and explaining placement are two tickets.** 041 is the
  app-id, which is code: one owned string passed through 042's `app_id`
  arguments, with no Wayland call involved. The explanation of why a
  compositor rule and not Squire does the placing is README prose, and it
  waits for the README to exist, so it moved to
  `issues/045-readme-says-how-to-place-the-window.md`. Split 2026-08-26.

- **The app id is one owned name per view kind.** `command_line` fills `{id}`
  from a view kind, and the HUD's is `goldbox-squire-hud`. A compositor rule is
  written by hand once and breaks silently if the string drifts, so Squire owns
  every name and no caller can invent one. 041 first pinned a single constant,
  `goldbox-squire`, because "there was never more than one name to pass"; ADR
  0005 gives the map and the journal windows of their own, so that reason
  expired and 046 puts the parameter back typed. See
  `issues/041-name-the-window.md` and `issues/046-one-app-id-per-view.md`.

- **One host reads, many views draw.** `gbs` keeps the emulator, stays its
  parent and reads it with `process_vm_readv`, and hands the party out over a
  unix socket. Each window is a view: it draws what it is sent and sends the
  user's decisions back. A window can never read the emulator itself, because
  Yama permits a read of a descendant and only one process can be DOSBox's
  parent, so a second window would need the numbers shipped to it whatever
  happened. Inverting who launches was rejected for that reason, and passing
  an open `/proc/<pid>/mem` was rejected for costing `process_vm_readv`. The
  035 seams, `Screen` and `Keys`, become the wire in both directions.
  Settled 2026-08-26. See
  [ADR 0005](../../docs/adr/0005-one-host-reads-many-views-draw.md), and
  `issues/043-the-host-serves-the-party-on-a-socket.md`,
  `issues/046-one-app-id-per-view.md`,
  `issues/047-a-view-draws-what-the-socket-sends.md`,
  `issues/048-the-host-spawns-the-hud-and-becomes-the-log.md` and
  `issues/049-keys-travel-from-the-view-back-to-the-host.md`.

## Not yet specified

- **What the roomy sizes hold.** Map, journal and combat were tabs in the
  spike and are placeholders here. Nothing is designed for data Squire cannot
  yet read.
- **A command menu behind a key**, GBC style, for when there are enough
  commands to need one. Reserved, not designed.
- **Whether the wordmark survives.** It is drawn in the roomy mockup and
  nowhere else. Jeff has not said keep or cut. Nothing is blocked either way:
  036 asserts that it appears only when roomy, and cutting it later deletes an
  assertion rather than changing a rule.
- **How toggles are asked for once there is more than one.** Ability scores
  are the first thing with an on and off, and one key each will not scale when
  THAC0, experience and encumbrance land. Not worth designing before there is
  a second one.
- **Whether the table drawings go.** `.scratch/squire-hud/mockups/table-*.txt`
  and `mock.py` are the first pass, kept only in case the cards read worse in
  a real terminal than they look on paper. Jeff deletes them when he is sure.

## Out of scope

- **Automatic window placement.** Wayland forbids it. The compositor rule is
  the answer and it belongs in the README.
- **Pinning the HUD to the DOSBox window.** Same reason. Recorded in
  `docs/roadmap.md` already.
- **Any layout named after a size.** See the decisions above.
