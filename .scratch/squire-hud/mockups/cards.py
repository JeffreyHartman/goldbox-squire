#!/usr/bin/env python3
"""Draws the party as cards, at every size, as exact character grids.

One card per character. The card is the unit, not a table row. A card shrinks
by dropping lines, and the cards themselves either stack down the side or sit
across the top. That is the whole layout.

Throwaway. Ticket 034 asks for a decision, not for code that survives, and none
of this is a plan for the Rust module in 036.

    python3 cards.py

Every file it writes is exactly its stated rows by its stated columns and holds
nothing else, so it can be `cat` into a terminal of that size without wrapping
or scrolling. To check: awk '{print length}' FILE | sort -u prints one number.
"""

from dataclasses import dataclass, field


@dataclass
class Char:
    name: str
    klass: str
    level: int
    hp: int
    hp_max: int
    ac: int
    thac0: int
    # Everything currently on the character. Squire reads one of these today,
    # from the status byte. The rest of the list is what the effects read will
    # add later, and the card is shaped so that it can.
    effects: list
    abilities: tuple  # str int wis dex con cha
    # Percentile strength, for the fighters that rolled it. None for everyone
    # else. It has to survive the slash form, which is why it is in brackets.
    str_pct: int = None


PARTY = [
    Char("THRENDER GRONE", "fighter", 5, 42, 42, 2, 16, ["okay"], (18, 9, 11, 16, 17, 10), 72),
    Char("BROTHER SEAN", "cleric", 4, 26, 26, 4, 18, ["okay"], (14, 10, 17, 12, 15, 13)),
    Char("AMRYL", "mage", 4, 14, 14, 8, 19, ["okay"], (9, 18, 12, 16, 11, 14)),
    Char("KEIRA", "fighter/thief", 5, 18, 31, 5, 17, ["okay"], (16, 13, 10, 18, 14, 12)),
    Char("DURIN STONEFOOT", "fighter", 5, 38, 44, 1, 16, ["poisoned"], (17, 8, 12, 13, 18, 9)),
    Char("ELANNA", "cleric/mage", 3, 20, 22, 6, 19, ["okay"], (11, 17, 16, 14, 12, 15)),
]

GAME = "Pool of Radiance"
SLOT = "J"

# The narrowest card worth drawing. A name is fifteen characters and `hp
# 38/44` is eight, so anything under this cannot hold a whole line of either.
CARD_MIN = 16
# The narrowest card that is worth putting six of side by side. Below this the
# cards across the top are worse than the same cards down the side.
STRIP_MIN = 20
# The longest a hit point bar gets. Past this it stops telling you anything
# more and starts eating the line.
BAR_MAX = 12
# The shortest a bar can be and still mean anything. Below this it is a couple
# of blocks that barely move, so the numbers go alone instead.
BAR_MIN = 4


def bar(cur, top, width):
    """A hit point bar. Full blocks for what is left, light for what is gone."""
    if width < BAR_MIN or top <= 0:
        return ""
    filled = max(0, min(width, round(width * cur / top)))
    # A living character never draws an empty bar, so that "hurt" and "down"
    # never look the same at a glance.
    if cur > 0:
        filled = max(1, filled)
    return "█" * filled + "░" * (width - filled)


def fit(text, width):
    return text if len(text) <= width else text[: width - 1] + "…"


@dataclass
class Shape:
    """The widest each line gets across the whole party.

    Held so that every card in one party is laid out the same way, whatever
    the length of one character's name.
    """
    name: int
    klass: int
    abil: int


def shape_of(party):
    return Shape(
        name=max(len(c.name) for c in party),
        klass=max(len(f"{c.klass} · lvl {c.level}") for c in party),
        abil=max(len(abilities_line(c)) for c in party),
    )


def abilities_line(c):
    """Six numbers in the order every Gold Box screen prints them.

    No labels. A player who wants these knows the order, and six abbreviations
    cost more of the card than the numbers do.
    """
    first = f"{c.abilities[0]}({c.str_pct})" if c.str_pct else str(c.abilities[0])
    return "/".join([first] + [str(v) for v in c.abilities[1:]])


def card_lines(c, w, budget, shape=None, abilities=False):
    """One card's lines, widest first, cut to `budget` lines.

    The order the lines leave in is the settled drop order: name last, then
    hit points, then what is on the character, then class, level, armour
    class, then ability scores. What differs from a table is that a line that
    does not fit beside another simply gets its own.
    """
    klass_lvl = f"{c.klass} · lvl {c.level}"
    lines = []          # (priority, text). Lower survives longer.

    # Name, with the class and level pushed to the right when they fit.
    #
    # The test uses the longest name and the longest class in the party, not
    # this card's own. Every card in a party gets the same shape; one card
    # laid out differently because its owner has a short name reads as a bug.
    want = shape.name + 3 + shape.klass if shape else len(c.name) + 3 + len(klass_lvl)
    if w >= want:
        gap = w - len(c.name) - len(klass_lvl)
        lines.append((0, c.name + " " * gap + klass_lvl))
        klass_on_own_line = False
    else:
        lines.append((0, fit(c.name, w)))
        klass_on_own_line = True

    # Hit points, the bar, and armour class if the line still has room.
    hp = f"hp {c.hp}/{c.hp_max}"
    ac = f"ac {c.ac}"
    room = w - len(hp) - 1
    if room >= len(ac) + 3:
        b = bar(c.hp, c.hp_max, min(BAR_MAX, room - len(ac) - 2))
        gap = w - len(hp) - (1 if b else 0) - len(b) - len(ac)
        lines.append((1, f"{hp}{' ' if b else ''}{b}{' ' * gap}{ac}"))
        ac_on_own_line = False
    else:
        b = bar(c.hp, c.hp_max, min(BAR_MAX, max(0, room)))
        lines.append((1, f"{hp} {b}".rstrip()))
        ac_on_own_line = True

    # What is on the character. One line per effect, so that a character with
    # five of them is five lines and nothing is hidden.
    for e in c.effects:
        lines.append((2, fit(e, w)))

    if klass_on_own_line:
        lines.append((3, fit(f"{c.klass} {c.level}", w)))
    if ac_on_own_line:
        lines.append((4, ac))

    # Ability scores are off unless the user asks for them. They barely change
    # during play, and six numbers on every card makes the party look busy for
    # information nobody is watching. There is no half-measure: a card either
    # shows all six or none, because one number is not worth a line.
    abil = abilities_line(c)
    if abilities and (shape.abil if shape else len(abil)) <= w:
        lines.append((5, abil))

    keep = sorted(lines, key=lambda p: p[0])[:budget]
    kept = [t for _, t in sorted(keep, key=lambda p: lines.index(p))]
    return [t.ljust(w) for t in kept]


def card_wants():
    """The width a card is happiest at.

    Its widest line is the name with the class and level pushed to the right:
    the longest name in the party, a gap, and the longest class and level. Add
    the card's own two spaces of padding, and four more so that the text is
    not jammed against the frame.

    Every number here comes from the party and from the game's class names.
    None of it comes from anybody's monitor. The four spaces of air are the
    one judgement call, and widening or narrowing them is what moves a size
    like 80x40 between one column of cards and two.
    """
    s = shape_of(PARTY)
    return s.name + 3 + s.klass + 2 + 4
# The most lines a card ever has: name, hit points, one condition, abilities.
CARD_TALL = 5


def layout(cols, rows, abilities=False):
    """How many cards across, and how wide and tall each one is.

    Six cards go across, or down, or into a grid in between. The rule picks
    the number of columns that gets each card closest to the width it wants.
    Rows are a hard limit: a window too short for two rows of cards gets one
    row of six, whatever the width says.

    There are no named layouts here. `strip` and `sidecar` are what the ends
    of this range look like, not modes the code can be in.
    """
    body = rows - 2                      # the header and the footer
    best = None
    for across in (1, 2, 3, 6):
        down = len(PARTY) // across
        w = (cols - (across + 1)) // across - 2
        h = (body - (down + 1)) // down
        if w < CARD_MIN or h < 2:
            continue
        tall = max(len(card_lines(c, w, 99, shape_of(PARTY), abilities)) for c in PARTY)
        h = min(h, tall)
        miss = abs(w - card_wants())
        if best is None or miss < best[0]:
            best = (miss, across, down, w, h)
    return None if best is None else best[1:]


def grid(cols, rows, header, footer, across=None, abilities=False):
    """The party as a grid of cards."""
    chosen = layout(cols, rows, abilities)
    if chosen is None:
        return None
    if across is None:
        across, down, w, h = chosen
    else:
        down = len(PARTY) // across
        w = (cols - (across + 1)) // across - 2
        h = min(
            max(len(card_lines(c, w, 99, shape_of(PARTY), abilities)) for c in PARTY),
            (rows - 2 - (down + 1)) // down,
        )
        if w < CARD_MIN or h < 2:
            return None

    # Any columns the division left over go to the leftmost cards, so the
    # frame reaches the right edge exactly.
    spare = cols - (across + 1) - across * (w + 2)
    widths = [w + (1 if i < spare else 0) for i in range(across)]
    shape = shape_of(PARTY)

    def edge(left, mid, right):
        return left + mid.join("─" * (x + 2) for x in widths) + right

    out = [header, edge("┌", "┬", "┐")]
    for r in range(down):
        if r:
            out.append(edge("├", "┼", "┤"))
        row_cards = PARTY[r * across : (r + 1) * across]
        cells = [
            card_lines(c, x, h, shape, abilities) for c, x in zip(row_cards, widths)
        ]
        for cell, x in zip(cells, widths):
            while len(cell) < h:
                cell.append(" " * x)
        for line in range(h):
            out.append("│" + "".join(f" {cell[line]} │" for cell in cells))
    out.append(edge("└", "┴", "┘"))

    # Room to spare is where the wordmark goes, and nowhere else. What is left
    # below is where a map or a journal will live, if they ever do, so nothing
    # is drawn there.
    left = rows - 1 - len(out)
    if left >= WORDMARK_ROWS + 2:
        out += [""] * ((left - WORDMARK_ROWS) // 2) + centre(wordmark(), cols)
    while len(out) < rows - 1:
        out.append("")
    return out[: rows - 1] + [footer]


FONT = {
    "S": ["█████", "█    ", "█████", "    █", "█████"],
    "Q": ["█████", "█   █", "█ █ █", "█  ██", "█████"],
    "U": ["█   █", "█   █", "█   █", "█   █", "█████"],
    "I": ["█████", "  █  ", "  █  ", "  █  ", "█████"],
    "R": ["█████", "█   █", "█████", "█  █ ", "█   █"],
    "E": ["█████", "█    ", "█████", "█    ", "█████"],
}
WORDMARK_ROWS = 5


def wordmark():
    return ["  ".join(FONT[ch][r] for ch in "SQUIRE") for r in range(WORDMARK_ROWS)]


def centre(lines, cols):
    block = max(len(l) for l in lines)
    pad = " " * ((cols - block) // 2)
    return [pad + l for l in lines]


def draw(cols, rows, across=None, abilities=False):
    """The whole screen at this size."""
    left = f"gbs — {GAME} · slot {SLOT}"
    right = "watch"
    header = fit(left, cols) if len(left) + len(right) + 2 > cols else (
        left + " " * (cols - len(left) - len(right)) + right
    )
    footer = fit("1 Party · live · 6/6 · gold 1,240", cols)

    plan = grid(cols, rows, header, footer, across, abilities)
    if plan is None:
        return None
    screen = [line[:cols].ljust(cols) for line in plan]
    assert len(screen) == rows, (cols, rows, len(screen))
    assert all(len(l) == cols for l in screen)
    return screen


SIZES = [
    # cols, rows, label, cards across (None lets the rule decide), abilities
    (40, 20, "hostile", None, False),
    (50, 40, "sidecar-50", None, False),
    (60, 40, "sidecar-60", None, False),
    (80, 40, "sidecar-80", None, False),
    (110, 50, "tall", None, False),
    (160, 14, "wide-3across", None, False),
    (160, 42, "roomy", None, False),
    # Six across is what Jeff drew, and he wants it kept as a choice rather
    # than as the rule's answer. A key asks for it; the rule picks the rest.
    (160, 14, "wide-6across", 6, False),
    # The same layouts with the ability scores turned on, which is what the
    # key does. Nothing else about the card moves.
    (60, 40, "sidecar-60-abilities", None, True),
    (110, 50, "tall-abilities", None, True),
]


def main():
    import pathlib
    here = pathlib.Path(__file__).parent
    for cols, rows, label, across, abilities in SIZES:
        drawn = draw(cols, rows, across, abilities)
        if drawn is None:
            print(f"{label}: nothing fits at {cols}x{rows}")
            continue
        (here / f"cards-{cols}x{rows}-{label}.txt").write_text("\n".join(drawn) + "\n")
        chosen = across or layout(cols, rows, abilities)[0]
        print(f"{label}: {cols}x{rows}, {chosen} across, {len(PARTY) // chosen} down")


if __name__ == "__main__":
    main()
