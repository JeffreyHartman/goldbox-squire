# 049 — Keys travel from the view back to the host

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 047

## What to build

The user types in the view's window and the host acts on it. Quit ends the
run. Enter picks a different save slot.

The repick runs in the view, not the host. The wizard prints a menu and reads
a line, and the user is looking at the view's window. The view sends back the
slot letter and the names it resolved, so the host still owns the retarget and
there is still one wizard rather than two that can disagree.

Keys that only change what is drawn stay in the view and never reach the
socket. Moving the highlight is not the host's business.

## Acceptance criteria

- [ ] q, Escape and Ctrl-C in the view end the run
- [ ] Enter in the view asks the slot question in the view's window, and the
      host retargets to the answer
- [ ] The highlight, the ability toggle and the cards-across key stay in the
      view
- [ ] A view that dies mid-repick does not wedge the host
