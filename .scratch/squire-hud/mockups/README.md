# Mockups: the party at four sizes

Ticket 034. These are character grids, not pictures of them. Each file holds
the grid and nothing else: exactly the rows it names, each exactly the columns
it names, no caption and no ruler. A caption would wrap in a terminal sized to
the mockup, and a wrapped caption makes an exact grid look like a broken one.

## How to look at them

Resize a terminal to the stated size and `cat` the file into it. It should fill
the window with nothing wrapped and nothing scrolled off the top.

    awk '{print length}' 110x50-tall.txt | sort -u

That prints one number, and the number is the width. It is the check that these
are grids rather than pictures of grids.

The five files:

| File | Size | What survived |
| --- | --- | --- |
| `40x20-hostile.txt` | 40x20 | name, hit points, status. Narrow, with rows to spare |
| `40x20-hostile-cut-names.txt` | 40x20 | the same, plus class, at the cost of whole names |
| `160x14-short-and-wide.txt` | 160x14 | everything. Above or below the DOSBox window |
| `110x50-tall.txt` | 110x50 | everything, and the wordmark. Beside it |
| `160x42-roomy.txt` | 160x42 | everything, and the wordmark. Room to spare |

The sizes are scratch values chosen to span tall, roomy, short-and-wide, and
hostile. No number in this folder belongs in the code, and 036 must not copy
one.

`mock.py` draws them. Change the drop order in `COLUMNS`, rerun `python3
mock.py`, and all five redraw. It is throwaway: it is not a plan for the Rust
module.

## What the mockups assume

The party data is one plausible six-character party. KEIRA is wounded at 18 of
31. DURIN STONEFOOT is poisoned. Two characters are multiclassed, because
`fighter/mage/thief` is the longest class string the games can produce and it
is what sets the class column's width.

Every column is as wide as the widest value that field can ever hold. Fifteen
for a name, eighteen for a class, eleven for `unconscious`. Those come from the
game data, not from a screen.

## What I need you to decide

**1. The drop order.** The map settled: name, hit points, status, class, level,
armor class, ability scores, with the name last to go. The mockups obey it. At
40 columns that costs armor class, level, and class, in that order. Confirm it
or reorder it.

**2. Whole names, or one more field.** At 40 columns the table has room for
either full names or an abbreviated class column, not both. The two hostile
files are that choice. `40x20-hostile.txt` keeps every name whole and shows
name, hit points, and status. `40x20-hostile-cut-names.txt` cuts DURIN
STONEFOOT to `DURIN STONEFO…` and buys back the class. I lean toward cutting
the name, because you know your own party's names and you do not know its armor
class, but this is exactly the sort of call that should be yours.

**3. The one-column status.** When the word does not fit, status becomes one
glyph: `·` for okay and `!` for anything else. That is cheap and unmissable,
and it tells you something is wrong without telling you what. The alternative
is a glyph per status, which needs a legend you would have to learn. I would
keep the single `!`.

**4. The roomy threshold.** My proposal, and the one 036 will encode unless you
change it: it is roomy when every field fits, and the rows left over after the
party block, the rule and the status line are at least as many as the party
block itself uses, plus the wordmark's five rows and a gap. In words: roomy
means there is a whole second panel's worth of room going spare. It names no
monitor and no breakpoint, and it is why 160x14 gets no wordmark while 110x50
does.

**5. Centred, or stretched.** The party block is centred, and no column ever
grows past its widest possible value. The alternative is stretching the table
to fill the width, which at 160 columns turns `LVL` into fourteen columns of
air. I think centring is right, but it does leave a lot of margin at 160x14.

**6. Rows for columns, not built.** `40x20-hostile.txt` throws away eight rows
while dropping fields for want of columns. A layout that gave each character
two rows would fit everything at 40 columns. I did not build it, because it is
a second shape for the party rather than a different set of fields in one
shape, and that is a bigger decision than this ticket asked for. Say if you
want to see it.

## Not mocked up

No map, no journal, no combat. Nothing is drawn for data Squire cannot yet
read, which is the mistake that produced the spike's sixteen-column menu of
placeholder tabs. In the roomy files the room to spare is left empty on
purpose. That empty area is the point.
