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

One card per character. The card is the unit, not a row in a table.

    THRENDER GRONE                     fighter · lvl 5     <- name, class, level
    hp 42/42 ████████████                         ac 2     <- hit points, bar, armour class
    okay                                                   <- what is on them, one line each

The class and level sit on the name line when they fit, and get their own line
when they do not. Same for armour class and the hit point line. The bar is
dropped rather than drawn four cells wide, because a bar that short never
visibly moves.

The shape is decided once for the whole party, from the longest name and the
longest class in it. One card laid out differently because its owner has a
short name reads as a bug.

## Ability scores are a key, not a rule

They are off in every mockup except two. They barely change during play, and
six numbers on every card makes the party look busy for information nobody is
watching. There is no half measure: a card shows all six or none, because one
score is not worth a line.

When the key is pressed, one line appears above nothing and below the
conditions:

    18(72)/9/11/16/17/10

No labels. The order is the one every Gold Box screen prints, and a player who
wants these numbers knows it. `18(72)` is percentile strength, which has to
survive the slashes, so it goes in brackets.

    cards-60x40-sidecar-60-abilities.txt
    cards-110x50-tall-abilities.txt

Compare each against the file without `-abilities`. Nothing else about the card
moves.

This is a real toggle, not a width rule. That matters for 036: the layout plan
answers "does this fit", and the toggle answers "did the user ask for it".
Those are two different questions and the module has to take the toggle as an
input rather than deciding it.

## The layout

Six cards, arranged one across, two across, three across, or six across. There
is no strip mode and no sidecar mode. The rule picks the number across that
gets each card closest to the width it wants, and the rows are a hard limit: a
window too short for two rows of cards gets one row of six whatever the width
says.

What the card wants is worked out from the party: the longest name, a gap, the
longest class and level, the card's own padding, and four spaces of air. Every
part of that comes from the game data except the four spaces, and those four
are the one number worth arguing about. Widening them moves 80x40 from two
columns to one.

A key can ask for a different number across, which is how 160x14 gets both
answers.

`sidecar` and `strip` are what the ends of that range look like. They are not
states the code can be in.

| File | Size | What the rule chose |
| --- | --- | --- |
| `cards-40x20-hostile.txt` | 40x20 | 2 across, 3 down |
| `cards-50x40-sidecar-50.txt` | 50x40 | 1 across, 6 down |
| `cards-60x40-sidecar-60.txt` | 60x40 | 1 across, 6 down |
| `cards-80x40-sidecar-80.txt` | 80x40 | 2 across, 3 down |
| `cards-110x50-tall.txt` | 110x50 | 2 across, 3 down |
| `cards-160x14-wide-3across.txt` | 160x14 | 3 across, 2 down, what the rule picks |
| `cards-160x14-wide-6across.txt` | 160x14 | 6 across, what a key asks for |
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

## Settled

Cards, not tables. 60 is the sidecar width Squire starts at. Both wide answers
are kept and a key chooses. Ability scores are off by default, shown as six
slashed numbers when a key asks.

## What is left to decide

**1. The wordmark.** It appears only in `cards-160x42-roomy.txt`, in the room
going spare. Keep it or cut it.

**2. What else the key toggles.** Ability scores are the first thing with an
on and off. THAC0, experience, and encumbrance will want the same treatment
when they land, and a key each does not scale. Worth one thought now, but not
worth designing before there is a second one.

**3. Whether the tables go.** `table-*.txt` are the first pass, kept only in
case the cards read worse in a real terminal than they look here. Say the word
and `mock.py` and its five files go.

## Not mocked up

No map, no journal, no combat. In `cards-160x42-roomy.txt` the room to spare is
left empty on purpose. That empty area is where those would go, and it is the
point.
