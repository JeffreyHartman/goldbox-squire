# 031 — GOG's flat installs: the root is the game folder

Type: `wayfinder:task`
Status: open

## Question

Every current GOG build except Pool of Radiance puts the DOS files flat in
the install root: no `CURSE` folder, the mounted C: is the game directory,
saves in a root-level `SAVE\` folder (research 004). Two pieces of Squire
assume a folder named like the game's own DOS folder:

- Discovery walks for a directory named `game_folder`, so a flat GOG
  install of Curse is never found.
- The computed autoexec mounts the parent of `game_folder` and enters it,
  so even a manually pointed flat install refuses to launch: there is no
  `CURSE` component in the recorded save path.

The Steam installs nest the classic SSI names (`GAME\CURSE`), so they work
today. Fixing GOG needs a second install shape: "the root is the game
folder", detected by the start file plus save files sitting beside the
conf, mounted as `mount c <root>` with no `cd`.

No GOG install of these games is on this machine, so the work waits for
one, or for a tester who has one.
