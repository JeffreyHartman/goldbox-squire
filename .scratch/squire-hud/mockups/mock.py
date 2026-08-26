#!/usr/bin/env python3
"""Draws the party panel at four sizes, as exact character grids.

Throwaway. It exists so that changing the drop order is one edit and a rerun,
not six hand-drawn tables. Ticket 034 asks for a decision, not for code that
survives, and none of this is a plan for the Rust module in 036.

    python3 mock.py

Writes one .txt per size beside this file. Each file is exactly its stated
number of rows, and every row is exactly its stated number of columns.
"""

from dataclasses import dataclass

# The six characters. One wounded (KEIRA), one with a status (DURIN).
@dataclass
class Char:
    name: str
    klass: str
    abbrev: str
    level: int
    hp: int
    hp_max: int
    ac: int
    status: str
    abilities: tuple  # str int wis dex con cha


PARTY = [
    Char("THRENDER GRONE", "fighter", "F", 5, 42, 42, 2, "okay", (18, 9, 11, 16, 17, 10)),
    Char("BROTHER SEAN", "cleric", "C", 4, 26, 26, 4, "okay", (14, 10, 17, 12, 15, 13)),
    Char("AMRYL", "mage", "M", 4, 14, 14, 8, "okay", (9, 18, 12, 16, 11, 14)),
    Char("KEIRA", "fighter/thief", "F/T", 5, 18, 31, 5, "okay", (16, 13, 10, 18, 14, 12)),
    Char("DURIN STONEFOOT", "fighter", "F", 5, 38, 44, 1, "poisoned", (17, 8, 12, 13, 18, 9)),
    Char("ELANNA", "cleric/mage", "C/M", 3, 20, 22, 6, "okay", (11, 17, 16, 14, 12, 15)),
]

GAME = "Pool of Radiance"
SLOT = "J"

# One column of the party table. `drop` is the position in the drop order:
# higher goes first. The name is never dropped, so it has no number.
COLUMNS = [
    # key, header, width, drop
    # Every width is the widest the field can ever be: fifteen for a Gold Box
    # name, eighteen for `fighter/mage/thief`, eleven for `unconscious`. None
    # of them comes from a screen.
    ("name", "NAME", 15, None),
    ("hp", "HP", 7, 6),
    ("status", "STATUS", 11, 5),
    ("status1", "!", 1, 5),
    ("class", "CLASS", 18, 4),
    ("class1", "CL", 5, 4),
    ("level", "LVL", 3, 3),
    ("ac", "AC", 3, 2),
    ("abilities", "STR INT WIS DEX CON CHA", 23, 1),
]

WIDTH = {key: w for key, _, w, _ in COLUMNS}
HEADER = {key: h for key, h, _, _ in COLUMNS}

# What each column holds, per character.
# The narrowest a name column is worth drawing. A name cut shorter than this
# stops telling one fighter from another. It is a readability floor, not a
# measurement of anybody's screen.
NAME_FLOOR = 9


def cell(c, key):
    if key == "name":
        w = WIDTH["name"]
        return c.name if len(c.name) <= w else c.name[: w - 1] + "…"
    if key == "hp":
        return f"{c.hp}/{c.hp_max}"
    if key == "status":
        return c.status
    if key == "status1":
        return "·" if c.status == "okay" else "!"
    if key == "class":
        return c.klass
    if key == "class1":
        return c.abbrev
    if key == "level":
        return str(c.level)
    if key == "ac":
        return str(c.ac)
    if key == "abilities":
        return " ".join(f"{a:>3}" for a in c.abilities)
    raise KeyError(key)


def table_width(keys):
    """A bordered table costs two spaces and a rule per column, plus one rule."""
    return 1 + sum(WIDTH[k] + 3 for k in keys)


def fields_for(cols, cut_names=False):
    """The widest set of fields that fits in `cols`, in the settled drop order.

    Full words win over abbreviations when both fit, which is why status and
    class each appear twice in COLUMNS.
    """
    wide = ["name", "hp", "status", "class", "level", "ac", "abilities"]
    narrow = {"status": "status1", "class": "class1"}
    keys = list(wide)
    while keys:
        if table_width(keys) <= cols:
            return keys
        # Cutting a name short can buy back a whole field. Whether that is a
        # good trade is the question 034 puts to Jeff, so both answers get a
        # mockup.
        if cut_names:
            need = table_width(keys) - cols
            if need <= WIDTH["name"] - NAME_FLOOR:
                WIDTH["name"] -= need
                return keys
        # Try the cheap spelling of anything that has one, before dropping.
        shrunk = [narrow.get(k, k) for k in keys]
        if shrunk != keys and table_width(shrunk) <= cols:
            return shrunk
        keys = shrunk
        # Drop the last surviving field in the drop order.
        order = ["abilities", "ac", "level", "class1", "class", "status1", "status", "hp"]
        for candidate in order:
            if candidate in keys:
                keys.remove(candidate)
                break
        else:
            return keys
    return keys


BOX = dict(tl="┌", tr="┐", bl="└", br="┘", h="─",
           v="│", tt="┬", bt="┴", lt="├", rt="┤",
           x="┼")


def rule(keys, left, mid, right):
    return left + mid.join(BOX["h"] * (WIDTH[k] + 2) for k in keys) + right


def row(keys, values):
    out = BOX["v"]
    for k in keys:
        out += " " + values[k].ljust(WIDTH[k]) + " " + BOX["v"]
    return out


def party_block(keys):
    lines = [rule(keys, BOX["tl"], BOX["tt"], BOX["tr"])]
    lines.append(row(keys, {k: HEADER[k] for k in keys}))
    lines.append(rule(keys, BOX["lt"], BOX["x"], BOX["rt"]))
    for c in PARTY:
        lines.append(row(keys, {k: cell(c, k) for k in keys}))
    lines.append(rule(keys, BOX["bl"], BOX["bt"], BOX["br"]))
    return lines


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


def status_line(cols, keys):
    dropped = [k for k in ["abilities", "ac", "level", "class", "status", "hp"]
               if k not in keys and (k + "1") not in keys]
    left = f"1 Party   {GAME} · slot {SLOT} · live"
    if len(left) > cols:
        left = f"1 Party · slot {SLOT} · live"
    if len(left) > cols:
        left = f"Party · live"
    return left[:cols]


def centre(lines, cols):
    """Centres the party block in the width.

    No column is ever wider than its widest possible value, so a wide window
    does not turn LVL into fourteen columns of air. What is left over is
    margin, and rows are where room to spare is spent.
    """
    block = max(len(l) for l in lines)
    pad = " " * ((cols - block) // 2)
    return [pad + l for l in lines]


def draw(cols, rows, label, cut_names=False):
    for key, _, w, _ in COLUMNS:
        WIDTH[key] = w
    keys = fields_for(cols, cut_names)
    body_rows = len(party_block(keys))
    # The spike's gold double rule, kept. It separates the party from the
    # status line and costs one row.
    # The gold double rule the spike had, kept: it separates the party from
    # whatever else is on screen and costs one row.
    spare = rows - body_rows - 2  # the double rule and the status line
    # Roomy: every field fits, and what is left is a whole second panel's
    # worth of rows, plus the wordmark's own five and its gap.
    roomy = ("abilities" in keys) and spare >= body_rows + WORDMARK_ROWS + 1
    block = centre(party_block(keys), cols)
    lines = []
    if roomy:
        lines += centre(wordmark(), cols)
        lines.append("")
    lines += block
    while len(lines) < rows - 2:
        lines.append("")
    lines = lines[: rows - 2]
    lines.append("═" * cols)
    lines.append(status_line(cols, keys))
    grid = [line[:cols].ljust(cols) for line in lines]
    assert len(grid) == rows
    assert all(len(l) == cols for l in grid)
    return keys, roomy, grid


SIZES = [
    (40, 20, "hostile", False),
    (160, 14, "short-and-wide", False),
    (110, 50, "tall", False),
    (160, 42, "roomy", False),
    # The alternative answer at the hostile size: cut the names, keep the
    # field. Ticket 034 asks Jeff which of these two he wants.
    (40, 20, "hostile-cut-names", True),
]


def main():
    import pathlib
    here = pathlib.Path(__file__).parent
    for cols, rows, label, cut_names in SIZES:
        keys, roomy, grid = draw(cols, rows, label, cut_names)
        shown = ", ".join(k.rstrip("1") for k in keys)
        head = [
            f"{label}: {cols} columns x {rows} rows",
            f"shown: {shown}" + ("  + wordmark" if roomy else ""),
            "",
            "─" * cols + "  <- exactly this wide",
        ]
        tail = ["─" * cols]
        text = "\n".join(head + grid + tail) + "\n"
        (here / f"{cols}x{rows}-{label}.txt").write_text(text)
        print(f"{label}: {shown}{' + wordmark' if roomy else ''}")


if __name__ == "__main__":
    main()
