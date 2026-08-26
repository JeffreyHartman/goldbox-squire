# Mockups: the party as cards

Ticket 034. These are character grids, not pictures of them. Each file holds
the grid and nothing else: exactly the rows it names, each exactly the columns
it names, no caption and no ruler.

## How to look at them

Resize a terminal to the stated size and `cat` the file into it. It should fill
the window with nothing wrapped and nothing scrolled off the top.

    awk '{print length}' cards-110x50-tall.txt | sort -u

That prints one number, and the number is the width.

## The card

One card per character. The card is the unit, not a row in a table. A card
holds up to five lines, and it drops them from the bottom as it shrinks:

    THRENDER GRONE                     fighter · lvl 5     <- name, class, level
    hp 42/42 ████████████                         ac 2     <- hit points, bar, armour class
    okay                                                   <- what is on them, one line each
    str 18 · int 9 · wis 11 · dex 16 · con 17 · cha 10     <- ability scores

The class and level sit on the name line when both fit, and get their own line
when they do not. Same for armour class and the hit point line. The bar is
dropped rather than drawn four cells wide, because a bar that short never
visibly moves.

Two things are decided for the whole party rather than per card. Whether the
class sits on the name line, and whether the ability line is the long form or
`str 18 · thac0 16`. One card laid out differently because its owner has a
short name reads as a bug.

## The layout

Six cards, arranged one across, two across, three across, or six across. There
is no strip mode and no sidecar mode. The rule picks the number across that
gets each card closest to the width it wants, and the rows are a hard limit: a
window too short for two rows of cards gets one row of six whatever the width
says.

`sidecar` and `strip` are what the ends of that range look like. They are not
states the code can be in.

| File | Size | What the rule chose |
| --- | --- | --- |
| `cards-40x20-hostile.txt` | 40x20 | 2 across, 3 down |
| `cards-50x40-sidecar-50.txt` | 50x40 | 1 across, 6 down |
| `cards-60x40-sidecar-60.txt` | 60x40 | 1 across, 6 down |
| `cards-80x40-sidecar-80.txt` | 80x40 | 2 across, 3 down |
| `cards-110x50-tall.txt` | 110x50 | 2 across, 3 down |
| `cards-160x14-wide.txt` | 160x14 | 3 across, 2 down |
| `cards-160x14-wide-forced-strip.txt` | 160x14 | 6 across, forced, for comparison |
| `cards-160x42-roomy.txt` | 160x42 | 3 across, 2 down, and the wordmark |

`python3 cards.py` redraws all of them.

## Statuses, and why the card holds a list

You were right and it is worse than you thought.

Squire reads **one** status byte per character today. One word, out of a list
of about ten. `okay`, `poisoned`, `unconscious`, `stoned`, and so on. That is
the only condition data in the character record.

GBC's green lines (`dwarf giant bonus`, `30% sleep/charm resist`,
`halfling poison bonus`) are not that byte. They live elsewhere in memory, in
what the roadmap calls conditions and effects, and Squire does not read that
place yet.

So the card holds a **list**, one line per item, sitting between the hit point
line and the ability line. Today the list has one item in it. When the effects
read lands, the same card looks like this and nothing else changes:

    DURIN STONEFOOT                    fighter · lvl 5
    hp 38/44 ██████████░░                         ac 1
    poisoned
    dwarf giant bonus
    dwarf save bonus
    halfling poison bonus
    str 17 · thac0 16

That is why the effects go above the ability line rather than below it. When
there are six of them, the ability scores are what should fall off the card,
not the thing that is currently killing you.

The mockups show one status each, because that is all Squire can read. Nothing
here is drawn for data Squire does not have.

## The table, kept

`table-*.txt` are the old one-table-for-everyone drawings, kept as the second
option in case the cards turn out worse in a real terminal than they look here.
`python3 mock.py` redraws those. `table-40x20-hostile-cut-names.txt` is the
answer you already gave for the table: cut long names, keep the field.

## What I still need you to decide

**1. Are cards right.** Compare `cards-60x40-sidecar-60.txt` against
`table-110x50-tall.txt`. If the cards are right, the table code goes.

**2. The sidecar width.** 50, 60, and 80 are drawn. 50 and 60 both give one
column of six cards, and 60 is the first width where the full ability line
fits. 80 flips to two columns of three, which may be a nice surprise or may be
wrong. I would make 60 the size Squire remembers on a first run.

**3. Six across, or three across two down, at 160x14.** The rule picks three
across because that gets each card near the width it wants. Six across is what
you drew. Both files are there. I lean toward letting the rule decide, because
the moment somebody has a 120 column window the six-across answer breaks and
three-across still works.

**4. The wordmark, still.** It appears only in `cards-160x42-roomy.txt`, in the
room going spare. Keep it or cut it.

## Not mocked up

No map, no journal, no combat. In `cards-160x42-roomy.txt` the room to spare is
left empty on purpose. That empty area is where those would go, and it is the
point.
