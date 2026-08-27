# ADR 0005 — One host reads, many views draw

Status: accepted
Date: 2026-08-26
Amends: ticket 011's process shape; ticket 041's single app id

## Context

ADR-less until now, ticket 011 settled that gbs launches DOSBox as its own
child, because Yama's descendant rule is what makes `process_vm_readv`
permitted with no system change. At `kernel.yama.ptrace_scope = 1`, the distro
default, only an ancestor of the target may read it.

Ticket 043 then asked for the HUD to move into its own window, so that a
compositor rule can place it beside the game. A window is a process. If gbs
keeps DOSBox and the HUD moves out, the HUD is DOSBox's sibling, and a sibling
is not an ancestor. The read is refused.

Two ways out were considered and rejected.

**Invert who launches**, so that the process in the new window starts DOSBox
and reads it, while gbs watches and prints the log. This works for one window
and cannot work for two. Only one process can be DOSBox's parent. The roadmap
already wants a map and a journal beside the HUD (`docs/roadmap.md`, "Maps and
views"), and the second window would need the numbers shipped to it anyway.

**Pass an open `/proc/<pid>/mem` descriptor** to the HUD. The kernel checks
the ptrace permission at open and caches the mm, so an inherited descriptor
keeps working. It costs `process_vm_readv`, which `mem.rs` measures at roughly
three times faster for a scan, and it is a trick rather than a design: nothing
in the code would say why it works.

## Decision

**gbs is the host. Every window is a view. One host, many views.**

The host launches DOSBox, stays its parent, and reads it with
`process_vm_readv`. Ticket 011's permission model is untouched. The host also
listens on a unix socket and hands the party out as data.

A view is a process in its own window. It connects to the socket, draws what
arrives, and sends the user's decisions back. A view never reads the
emulator's memory and never learns an address.

- **State on the wire, not pixels.** The host sends the party as data and each
  view renders it its own way. A map view wants a different shape of the same
  reading than the HUD does, so the host cannot render for it without learning
  every view's layout.
- **The socket is `$XDG_RUNTIME_DIR/goldbox-squire/<pid>.sock`.** Per run, not
  per user, so two games at once is not a special case. The runtime directory
  is cleared at logout, so a killed host leaves nothing behind.
- **Views are throwaway; the host is the run.** Closing a view changes
  nothing: the game plays on and the view can be opened again. Quitting the
  host closes every view. DOSBox is left running either way, per 011.
- **One app id per view kind**, `goldbox-squire-hud` and one for each view
  added after it, from a fixed list in the code. Ticket 041 pinned a single
  name on the reasoning that "there was never more than one name to pass".
  That reason expired here: the user writes one compositor rule per window,
  and one shared name would place the map wherever it places the HUD.

## Consequences

The two seams that ticket 035 cut out of the watch loop, `Screen` and `Keys`,
become the wire in both directions. The host's `Screen` writes to the socket
and its `Keys` reads from it. Nothing in the loop changes.

The slot repick runs in the view, not the host. The wizard prints a menu and
reads a line, and the user is looking at the view's window, not at the host's.
The view sends the resolved slot and names back, so the host still owns the
retarget.

`--plain` spawns nothing and opens no socket. A pipe is not a window.
