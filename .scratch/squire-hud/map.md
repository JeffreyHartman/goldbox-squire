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
- **Drop order, last to go first**: name, current and maximum hit points,
  status, class, level, armor class, ability scores. Status ranks high because
  a silent `poisoned` is the thing you most want to notice without looking away
  from the game, and it is cheap in columns as a symbol. Ability scores appear
  only when roomy, because they barely change during play.
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

## Not yet specified

- **What the roomy sizes hold.** Map, journal and combat were tabs in the
  spike and are placeholders here. Nothing is designed for data Squire cannot
  yet read.
- **A command menu behind a key**, GBC style, for when there are enough
  commands to need one. Reserved, not designed.

## Out of scope

- **Automatic window placement.** Wayland forbids it. The compositor rule is
  the answer and it belongs in the README.
- **Pinning the HUD to the DOSBox window.** Same reason. Recorded in
  `docs/roadmap.md` already.
- **Any layout named after a size.** See the decisions above.
