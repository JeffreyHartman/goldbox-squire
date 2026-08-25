# A DOS-native TUI module manager for FRUA: feasibility

Research spike. Question: could a DOS program, running inside DOSBox next to
FRUA itself, present a text UI that lists FRUA modules and downloads them from
the internet?

Everything below marked *verified locally* was checked against a file or a live
HTTP request from this machine, not inferred. Everything else is cited to the
project that owns the claim.

**Note:** `docs/` is gitignored in this repo. This file is a working note, not a
tracked artifact.

Last updated: 2026-08-23

---

## 1. Networking in DOSBox

### DOSBox Staging

Staging emulates a Novell NE2000 ISA card. Its only backend is libslirp; there
is no pcap backend in any release. `src/misc/ethernet.cpp` carries the comment
`// Currently only slirp is supported`, and `src/hardware/ne2000.cpp` hardcodes
`ethernet = ETHERNET_OpenConnection("slirp");`
(<https://github.com/dosbox-staging/dosbox-staging/blob/v0.82.2/src/misc/ethernet.cpp>,
<https://github.com/dosbox-staging/dosbox-staging/blob/v0.82.2/src/hardware/ne2000.cpp>).

There is no checked-in config template. Staging generates its config at runtime
from property registrations in `src/dosbox.cpp`. At the **v0.82.2** tag the
section is `[ethernet]` with exactly these settings and **no `backend=` key at
all** (<https://github.com/dosbox-staging/dosbox-staging/blob/v0.82.2/src/dosbox.cpp#L1238-L1320>):

| Setting | Default at v0.82.2 | Allowed |
|---|---|---|
| `ne2000` | **`true`** | bool |
| `nicbase` | `300` | 200/220/240/260/280/2c0/300/320/340/360 |
| `nicirq` | `3` | 3/4/5/9/10/11/12/14/15 |
| `macaddr` | `AC:DE:48:88:99:AA` | |
| `tcp_port_forwards` | unset | `21 80 443`, `8080:80`, `27910-27960` |
| `udp_port_forwards` | unset | same syntax |

**The v0.82.2 tarball in `~/Downloads` does support this**, confirmed
independently against the binary itself (*verified locally*,
`strings dosbox-staging-linux-x86_64-0.82.2-5e2ba/dosbox`):

- **libslirp v4.7.0 is statically vendored.** 135 slirp strings, with build paths
  like `../subprojects/libslirp-v4.7.0/src/tcp_input.c`. `ldd` shows no external
  slirp dependency, so nothing needs installing on the host.
- The NE2000 device is fully present: `NE2000: Initialised on port %xh and IRQ %u`,
  page 0/2 register handlers, `SlirpEthernetConnection`.
- All six config keys are present, and the help text is verbatim:

```
Enable emulation of a Novell NE2000 network card on a software-based
network (using libslirp) with properties as follows (enabled by default):
  - 255.255.255.0:  Subnet mask of the 10.0.2.0 virtual LAN.
  - 10.0.2.2:       IP of the gateway and DHCP service.
  - 10.0.2.3:       IP of the virtual DNS server.
  - 10.0.2.15:      First IP provided by DHCP, your IP!
Note: Inside DOS, setting this up requires an NE2000 packet driver, DHCP
      client, and TCP/IP stack.
```

libslirp is a Meson option that is on by default:
`option('use_slirp', type: 'boolean', value: true, ...)`, and `meson.build` sets
both `C_SLIRP` and `C_NE2000` from that one option, requiring
`dependency('slirp', version: ['>= 4.6.1', '< 5'])` with a subproject wrap
fallback (<https://github.com/dosbox-staging/dosbox-staging/blob/v0.82.2/meson_options.txt>).
If slirp is absent the whole section is registered inactive. Historically Windows
MSVC builds lacked it; the 0.80.1 notes say "The MSVC vs MSYS2 builds differ in
that the latter supports Ethernet networking"
(<https://github.com/dosbox-staging/dosbox-staging/releases/tag/v0.80.1>).

Version history, read off the tags:

- **v0.77.0** first shipped `src/hardware/ne2000.cpp`, but the section was
  `[ne2000]` with `realnic=` and help text "Enable Ethernet passthrough.
  Requires [Win]Pcap." `C_NE2000` came from `use_slirp or use_pcap`, both
  disabled by default.
- **v0.78.0** is where the current shape lands: `ethernet_slirp.cpp` appears, the
  section is renamed `[ethernet]`, the pcap/`realnic` path is removed, and the
  10.0.2.x help text arrives
  (<https://github.com/dosbox-staging/dosbox-staging/blob/v0.78.0/src/dosbox.cpp#L980-L1000>).
- 0.79 through 0.82 change nothing here.
- **0.83.0-RC1** moves the files to `src/network/` and **flips the default to
  off**: `AddBool("ne2000", WhenIdle, false)` with help "('off' by default)", and
  adds "Dynamically load Slirp library at runtime (#4330)"
  (<https://github.com/dosbox-staging/dosbox-staging/releases/tag/0.83.0-RC1>).
  The published manual agrees: `ne2000` default `off`
  (<https://www.dosbox-staging.org/0.83/manual/networking/ethernet/>).

**This matters for this repo.** `docs/findings/FINDINGS.md` records the working
emulator here as dosbox-staging 0.83.0-RC1 from the AUR, where `ne2000` defaults
to **off**. Whatever gbs writes into a game's conf must set `ne2000 = true`
explicitly rather than rely on a default that differs across the two versions on
this machine.

Staging **ships no packet driver and names none in its docs**. Its manual only
says "Setting this up requires installing a DOS packet driver, a DHCP client, and
a TCP/IP stack inside DOS."

### DOSBox-X

Same Bochs-derived NE2000 core, but with selectable backends. From its own
`dosbox-x.reference.conf`
(<https://github.com/joncampbell123/dosbox-x/blob/master/dosbox-x.reference.conf>):

```
[ne2000]
ne2000  = false
nicbase = 300
nicirq  = 3
macaddr = random
backend = auto      # pcap, slirp, ethnet, nothing, auto, none

[ethernet, pcap]
realnic = list
timeout = default

[ethernet, slirp]
ipv4_network = 10.0.2.0   ipv4_netmask = 255.255.255.0
ipv4_host    = 10.0.2.2   ipv4_nameserver = 10.0.2.3
ipv4_dhcp_start = 10.0.2.15
# plus restricted, disable_host_loopback, mtu, mru, tcp_port_forwards, udp_port_forwards
```

`backend=auto` prefers slirp, then pcap. IPX is separate and independent
(`[ipx] ipx = false`). Its wiki networking guide
(<https://github.com/joncampbell123/dosbox-x/wiki/Guide:Setting-up-networking-in-DOSBox%E2%80%90X>)
gives the trade: pcap needs promiscuous mode, so "applications in DOSBox-X can
use various legacy network protocols, such as IPX/SPX and NetBIOS Frames in
addition to TCP/IP", but "generally fails on wireless adapters"; slirp "does not
require Promiscuous mode ... will work with wireless adapters, and it will work in
most sandboxed environments" but "only supports the TCP/IP protocol".

The DOS-side sequence it documents is mTCP-based, which is the strongest
endorsement available for the toolchain choice in section 3:

```
SET MTCPCFG=C:\SAMPLES\SAMPLE.CFG
NE2000 0x60 10 0x300
DHCP
PING google.com
```

Timeline from its own release notes: pcap NE2000 in 0.83.7 (2020-11); slirp
backend and the `[ethernet, slirp]` / `[ethernet, pcap]` sections in **0.83.12**
(2021-04-01, <https://dosbox-x.com/release-0.83.12.html>); auto backend in
0.83.14; **bundled `NE2000.COM` in 0.83.15**; slirp port forwards in 0.83.21; an
experimental `ethnet` client/server backend in 2026.08.02.

DOSBox-X is the more capable fork here. For this job that capability is not
needed: all we want is outbound TCP, which slirp gives.

### Upstream DOSBox SVN / 0.74-3

**No NE2000, no ethernet, no slirp.** `src/hardware/` in the official SVN tree
contains `ipx.cpp` and `ipxserver.cpp` and nothing NE2000-related
(<https://sourceforge.net/p/dosbox/code-0/HEAD/tree/dosbox/trunk/src/hardware/>).
The wiki describes only tunnelling: "DOSBox emulates outdated protocols and
actually routes them to Internet's own IP protocol ... TCP/IP for serial/modem
emulation and UDP/IP for IPX emulation" (<https://www.dosbox.com/wiki/Connectivity>).
A DOS program cannot get a TCP/IP stack of its own there. Mainline is out.

### The DOS-side packet driver, and whether it can be bundled

**DOSBox-X bundles it, which settles the question of what to use.** Release
0.83.15 (2021-07-01): "Added NE2000.COM (packet driver for NE2000 network card)
which will appear in Z:\SYSTEM directory when NE2000 networking feature is
enabled. (Wengier)"
(<https://github.com/joncampbell123/dosbox-x/releases/tag/dosbox-x-v0.83.15>).
The binary is embedded as a byte array in
`src/builtin/ne2000bin.cpp` as `bfb_NE2000_COM`.

That embedded array (8693 bytes) is **byte-identical, md5
`0f4137af8babe643dff7aa6b1bcb0516`, to `NE2000.COM` inside Crynwr's own
`pktd11.zip`** from <http://crynwr.com/drivers/>. Its strings confirm it:

> `Packet driver for NE2000, version 11.4.3` ... `Packet driver skeleton
> copyright 1988-93, Crynwr Software. This program is freely copyable; source
> must be available; NO WARRANTY. See the file COPYING.DOC for details`

**Licence, and it has a wrinkle.** `COPYING.DOC` inside Crynwr's `pktd11.zip` is
the **GNU General Public License, Version 1, February 1989**, v1 not v2, and
with no "or any later version" clause. The archive's `READ.ME` summarises it:
"Anyone with a copy of the drivers may give it away, and the source code for all
modules must be available. NO WARRANTY". Crynwr's own site: "We give away all of
our packet drivers. They're OSI Certified Open Source software"
(<http://crynwr.com/>). Full v11.x sources are published as
`pktd11a.zip` / `pktd11b.zip` / `pktd11c.zip` at <http://crynwr.com/drivers/>.

So yes, it is redistributable, on GPLv1 terms: ship the binary plus the
corresponding source (or a written offer), and preserve the copyright and
no-warranty notices. Because it is an unmodified separate executable, the
copyleft obligation attaches to the driver, not to the tool that invokes it.
DOSBox-X, itself GPLv2, is the working precedent. DOSBox Staging bundles nothing
and names nothing, so a Staging-targeted tool has to supply the driver itself.

To match Staging's defaults, the invocation is `NE2000 0x60 3 0x300`: software
interrupt 0x60, IRQ 3, base 0x300.

### Does IPv4, DNS and DHCP work end to end through slirp?

Yes, for everything that matters here. Staging's `ethernet_slirp.cpp` at v0.82.2
sets `config.restricted = false` (with a comment that restricting "would cause
libslirp's internal DHCP server to fail"), `vnetwork = 10.0.2.0`,
`vhost = 10.0.2.2`, `vnameserver = 10.0.2.3`, `vdhcp_start = 10.0.2.15`, leaves
IPv6 off, and enables DHCPv4/BOOTP/TFTP.

- **DHCP: yes.** libslirp runs an internal DHCP server on the gateway. mTCP's
  `DHCP` command is the documented client, per the DOSBox-X guide.
- **DNS: yes**, a virtual resolver at 10.0.2.3 proxying to the host's resolver.
- **Arbitrary outbound TCP: yes, including 443.** libslirp is a user-mode NAT:
  outbound guest connections terminate in the library and are re-opened as
  ordinary host sockets, so any destination port works with no per-port
  configuration. Note the asymmetry: `tcp_port_forwards` is for **inbound**
  only. Its help text's example, "`21 80 443` ... This will forward FTP, HTTP,
  and HTTPS into the DOS guest", is about running a server in DOS, not about
  fetching. A downloader needs no forwards at all.
- **UDP: yes**, same NAT treatment.
- **ICMP: essentially no**, and this is a debugging trap. QEMU documents the same
  library's behaviour: "Note that ICMP traffic in general does not work with user
  mode networking. `ping`, aka. ICMP echo, to the local router (10.0.2.2) shall
  work" (<https://www.qemu.org/docs/master/system/devices/net.html>). So a DOS
  `PING` of an internet host can fail while TCP works fine. Do not use `PING` as
  the smoke test.
- **Non-IP protocols: no.** IPX/SPX and NetBIOS frames do not traverse slirp.
  Only DOSBox-X's pcap backend passes raw non-IP frames, and Staging has no pcap
  backend at all.
- No root and no promiscuous mode needed; libslirp is "a user-mode networking
  library" (<https://gitlab.freedesktop.org/slirp/libslirp/-/blob/master/README.md>).

**Outbound HTTPS works at the packet level.** Port 443 is not the obstacle. The
obstacle is that nothing in DOS real mode can do the TLS handshake, which is
section 2.

---

## 2. HTTPS on DOS

The question splits cleanly by memory model. In 16-bit real mode: no TLS exists,
full stop. In 32-bit DPMI: real TLS 1.3 exists and is maintained.

### mTCP has no TLS anywhere, and Brutman says so

mTCP ships DHCP, FTP, FTPSRV, HTGET, HTTPSERV, IRCJR, NC, NetDrive, PING,
PKTTOOL, SNTP and TELNET (<https://www.brutman.com/mTCP/mTCP.html>). Latest
release 10 January 2025, GPLv3.

Brutman's own manual, in the HTTPServ chapter under the heading "Enabling HTTPS
connections" (<https://www.brutman.com/mTCP/download/mTCP_2025-01-10.pdf>, p. 73):

> The mTCP HTTP server does not support HTTPS, but it may be desirable to allow
> for HTTPS connections. This can be done by using a reverse proxy, which takes
> the SSL connection and forwards it to the mTCP HTTP server without the
> encryption. Apache and other web servers can be used to implement a reverse
> proxy.

That is a proxy recommendation, not a TLS implementation. The source tree
confirms it: a case-insensitive grep of <https://github.com/mbbrutman/mTCP> for
`tls|ssl|https` matches only README URLs, a FAT spec link in
`src/APPS/NETDRV/REQHDR.H`, and a Unicode test-table URL in
`src/TCPLIB/UNICODE.CPP`. No crypto, no cipher code, no library hooks. The FTP
client is plain FTP, not FTPS. No release notes mention TLS.

**HTGET rejects `https://` at the argument parser.** In
`src/APPS/HTGET/HTGET.CPP` the URL parser tests only `strnicmp(url, "http://", 7)`,
defaults `ServerPort = 80`, and otherwise errors with `Need to specify a URL
starting with http://`. Worse for our purposes, **it does not follow redirects**:
on a 3xx it prints `New location: %s` and returns a DOS errorlevel (32 for 301,
33 for 302). So a server that forces an HTTP-to-HTTPS upgrade is a hard stop.

Brutman's own site footer makes his position explicit
(<https://www.brutman.com/>): "HTTPS is now available! ... The old, non-secure
method (HTTP) will still work because I don't want to break old computers that
can not use HTTPS." He keeps port 80 open *for* DOS clients.

Memory model, from `src/APPS/HTGET/MAKEFILE`: `memory_model = -ml` (large) with
`-0` (8088 codegen). Pure 16-bit real mode, no DPMI. That is exactly why TLS is a
non-starter on this path.

### Watt-32 has no TLS either, but it is what TLS gets ported onto

Watt-32 is "a library for making networked TCP/IP programs in the language of C
and C++ under DOS and Windows-NT. Both 16-bit real-mode and 32-bit
protected-mode is supported" (<http://www.watt-32.net/>). Source at
<https://github.com/gvanem/Watt-32>, last pushed 2026-06-24, so alive, even though
the website still advertises a 2018 release.

Grepping the repo for `openssl|mbedtls|bearssl|wolfssl` hits only license
boilerplate in the bundled 2002-era `bin/WGET.182/` sample. But its site states
the important part: "Watt-32 is officially supported in the OpenSSL library, cURL
ftp/http/https file retriever and the Web-browser Lynx."

### Real TLS on DOS does exist: DJGPP + Watt-32 + OpenSSL

This is the one path that genuinely works, and it is documented by OpenSSL and
curl themselves.

- **OpenSSL master still ships a DOS target.** `NOTES-DJGPP.md`
  (<https://raw.githubusercontent.com/openssl/openssl/master/NOTES-DJGPP.md>):
  "OpenSSL has been ported to DJGPP, a Unix look-alike 32-bit run-time
  environment for 16-bit DOS ... You also need to have the WATT-32 networking
  package installed before you try to compile OpenSSL." Configure with
  `./Configure no-threads --prefix=/dev/env/DJDIR DJGPP`. The target is
  `Configurations/50-djgpp.conf`, which links `-lwatt` from `$(WATT_ROOT)/lib`.
  OpenSSL's own caveat at the top of that file: "We can't make any commitment to
  support the DJGPP platform, and rely entirely on the OpenSSL community to help
  is fine tune and test."
- `NOTES-DJGPP.md` also flags an operational trap: OpenSSL on DJGPP wants a
  `/dev/urandom$` provided by a third-party DOS randomness driver such as
  `NOISE.SYS`. No entropy source, no TLS.
- **curl documents MS-DOS as a build target** in its own `docs/INSTALL.md`
  (<https://raw.githubusercontent.com/curl/curl/master/docs/INSTALL.md>), "MS-DOS"
  section, passing `WATT_ROOT=/path/to/djgpp/net/watt` with `--with-openssl`.
  Requires DJGPP 2.04+, and "Compile Watt-32 (and OpenSSL) with the same version
  of DJGPP."
- **Prebuilt, in the DJGPP project's own archive**
  (<http://www.delorie.com/pub/djgpp/current/v2tk/>): `ssl102ub.zip`, dated
  2019-12-31, manifest "OpenSSL 1.0.2u for DJGPP V2", ported by Juan Manuel
  Guerrero, alongside `wat3211b.zip` (Watt-32 2.2 dev.11). 1.0.2u gives TLS 1.2,
  not 1.3, and is long EOL.
- **TLS 1.3 exists**: <https://github.com/jwt27/openssl-djgpp> (J.W. Jagersma,
  also a credited Watt-32 contributor) carries a `djgpp-ppa` branch at OpenSSL
  **3.1.4** (Nov 2023), with Debian packaging producing `libssl-djgpp-dev`
  depending on `libwatt-djgpp-dev`, and real DJGPP porting commits ("Define
  WATT32_NO_OLDIES before including socket headers", "Use usleep() for
  ossl_sleep()").

**Nothing comparable exists for BearSSL, mbedTLS or wolfSSL on DOS.** No port
with a live repo or release notes was found. Real-mode TLS should be treated as
nonexistent.

### DOS browsers, as a cross-check

- **Arachne** states its own protocols as "HTTP, FTP, POP3, SMTP, Gopher. But not
  HTTPS/SSL/TLS" (<http://arachne.atspace.co.uk/>). FreeDOS's own news item
  announcing v1.99;GPL quotes maintainer Glenn McCorkle asking for help to add
  "HTTPS capability via WATT32 & OpenSSL"
  (<https://sourceforge.net/p/freedos/news/2021/12/arachne-web-browser-v199gpl/>).
  That request is the proof it does not have it.
- **Links (Twibright)** is the DOS browser that does do HTTPS, via exactly the
  DJGPP route. Its ChangeLog records the "DOS DJGPP port" on 24 Aug 2013
  (<http://links.twibright.com/download/ChangeLog>), and the DOS binaries
  directory is maintained through `links-2.30.exe` (28 Jul 2024) and ships
  `links.crt`, a 209 KB CA bundle, plus `libraries-for-links-2.10/` containing
  `libwatt.a` and `ssl101pb.zip`
  (<http://links.twibright.com/download/binaries/dos/>). The page's own caveat is
  that DOS binaries are "beta quality - there are stability problems".
- **MicroWeb** (<https://github.com/jhhoward/MicroWeb>), real-mode and built on
  mTCP with OpenWatcom 1.9, is explicit: limitations include "HTTP only", and
  under "HTTPS limitations": "TLS encryption is currently not supported which
  means that only HTTP servers can be accessed directly. There are some options
  for HTTPS sites: Use a proxy server such as retro-proxy which converts HTTPS to
  HTTP ... Use the FrogFind! web service". retro-proxy's own README says its job
  is to "bypass modern https, which requires encryption that vintage web browsers
  don't support" (<https://github.com/DrKylstein/retro-proxy>).

### What this means here

For this specific project the TLS question turns out to be **moot**, and that is
the happy accident that makes the whole idea viable. The UA File Archive serves
plain HTTP on port 80 with no redirect, and its HTTPS is broken anyway (section
4). So the 16-bit real-mode path, which cannot do TLS, is not disadvantaged at
all against the target host. A DOS client speaking HTTP/1.0 with a `Host:` header
gets the real bytes.

The exposure is future-tense: if DreamHost ever fixes that certificate and turns
on a forced HTTPS redirect, an mTCP-based client dies instantly, and HTGET will
not even follow the redirect to tell you why. That is the argument for a mirror
you control, not an argument for TLS on DOS.

---

## 3. Toolchain for the TUI

Four candidate stacks. Only two are both maintained and buildable on Linux
without proprietary software.

### Open Watcom V2

The live project is <https://github.com/open-watcom/open-watcom-v2>; the V2
README says of the original openwatcom.org that "only WEB site is up, all other
services ... is down for long time, it looks like it is dead". V2 itself is
active, with a commit dated 2026-08-23 at the time of checking.

It hosts on Linux as a command-line cross-compiler. The Getting Started manual
lists hosts as "DOS (command line), 32-bit OS/2 ..., Windows 3.x (IDE), Windows
95/98/Me, Windows NT/2000/XP upto Windows 11, Linux (command line)"
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/c_readme.html>). Target
selection is a compiler switch, and the wiki has a "Step by step setup and build
instructions for Linux" section (<https://github.com/open-watcom/open-watcom-v2/wiki/Build>).
One wrinkle: building the toolchain itself on Linux wants DOSBox available
(`OWDOSBOX`) to run the `wgml` documentation utility. That is a
build-the-compiler dependency, not a runtime one.

DOS targets are all present: "Creating 16-bit DOS Applications", "Creating
32-bit Phar Lap 386|DOS-Extender Applications", "Creating 32-bit DOS/4GW
Applications", with the royalty-free DOS/4GW extender bundled and DOS/4G,
CauseWay and Phar Lap TNT supported
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/cpguide.html>). All five
16-bit memory models exist as real libraries under `\WATCOM\LIB286\DOS`
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/cguide.html>).

**Its TUI story is thin.** No curses in the box. What it ships is `conio.h`
(`getch`, `cprintf`, `cputs`) and the Watcom Graphics Library, whose text
functions (`_outtext`, `_settextposition`, `_settextwindow`, `_settextcolor`) are
each marked "Classification: PC Graphics / Systems: DOS"
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/clib.html>). Text
windows, colour, cursor control. No widgets. You write the menus, list boxes and
dialogs yourself, or you bring PDCurses, which has a Watcom makefile.

### DJGPP

DJGPP is "a complete 32-bit C/C++ development system for Intel 80386 (and
higher) PCs running DOS" (<http://www.delorie.com/djgpp/>). Its FAQ chapter 2
says the memory "presents a flat address space with no segmentation (you can say
goodbye to far and huge pointers and to memory models)"
(<http://www.delorie.com/djgpp/v2faq/faq2.html>).

**A DPMI host is mandatory at runtime.** FAQ 2: "Starting from v2.0, DJGPP
programs do not need a separate extender program, only a DPMI server to run."
FAQ 6.2, on "GCC says 'No DPMI'": "You don't have a DPMI server installed, and
DJGPP v2 requires it to run", with free CWSDPMI (`csdpmi*.zip`) as the remedy
(<http://www.delorie.com/djgpp/v2faq/faq6_2.html>,
<http://sandmann.dotster.com/cwsdpmi/>). So a DJGPP binary ships with a second
file next to it.

**DJGPP has by far the best TUI story of the C toolchains**, because the
official archive carries both libraries prebuilt
(<http://www.delorie.com/pub/djgpp/current/v2tk/00_index.txt>):

```
pdcur39a.zip   171,469  2019-12-30   PDCurses 3.9 headers and libraries for DJGPP V2
tv210b.zip     604,315  2008-04-12   Turbo Vision - C++ Text User Interface library for DJGPP V2
tv210s.zip   1,706,438  2008-04-12   Turbo Vision - C++ Text User Interface library sources
```

PDCurses' own DOS README confirms the DJGPP port ("DJGPP port was provided by
David Nugent") and the top-level README calls it "a public domain curses library
for DOS, OS/2, Windows console, X11 and SDL"
(<https://github.com/wmcbrine/PDCurses/blob/master/dos/README.md>,
<https://github.com/wmcbrine/PDCurses>). There is **no ncurses port** anywhere in
the current DJGPP archive indexes. Cross-compiling from Linux is documented
first-party in FAQ 22.9 (<http://www.delorie.com/djgpp/v2faq/faq22_9.html>).

### Free Pascal plus Free Vision

FPC's platform list covers i8086 DOS and, under i386, the "GO32V2 DOS extender"
(<https://wiki.freepascal.org/Platform_list>). **go32v2 is the mature target**;
the wiki's DOS page flags i8086-msdos as newer and notes that on it "data
structures cannot be larger than 64KB" (<https://wiki.freepascal.org/DOS>).

Free Vision is the real thing: a Turbo Vision-compatible framework "included with
the FPC source package", and what the Free Pascal IDE's own interface is built
on. The package's `Makefile.fpc.fpcmake` lists units `app dialogs drivers editors
gadgets menus msgbox statuses stddlg tabs validate views outline`, i.e. a full
widget inventory, and `fpmake.pp` includes both `go32v2` and `msdos` in its OS
set (<https://gitlab.com/freepascal.org/fpc/source/-/blob/main/packages/fv/Makefile.fpc.fpcmake>,
<https://gitlab.com/freepascal.org/fpc/source/-/blob/main/packages/fv/fpmake.pp>).
In-tree working examples exist: `testapp.pas`, `filedlg.pas`, `demoedit.pas` and
others (<https://gitlab.com/freepascal.org/fpc/source/-/tree/main/packages/fv/examples>).
It "is not 100% complete": the `colorsel` unit is missing, palette support is
incomplete (<https://wiki.freepascal.org/Free_Vision>).

**And it has no networking at all on DOS.** `packages/rtl-extra/fpmake.pp`
defines `SocketsOSes = UnixLikes+AllAmigaLikeOSes+[netware,netwlibc,os2,emx,wince,win32,win64]`.
`go32v2` and `msdos` are absent, so the `Sockets` unit is not even built for DOS
targets (<https://gitlab.com/freepascal.org/fpc/source/-/blob/main/packages/rtl-extra/fpmake.pp>).
Reaching mTCP or Watt-32 from FPC means hand-writing external declarations
against a C-built library and matching the calling convention and memory model
yourself. Not a supported path.

So Free Pascal has the nicest TUI and the worst networking. Exactly inverted
from what this project needs.

### Turbo Vision C++ revivals

Borland did release the source publicly; magiblot/tvision's `COPYRIGHT` opens
"Borland International made the Turbo Vision source code public, accompanied by
the following disclaimer", with magiblot's own modifications MIT
(<https://github.com/magiblot/tvision/blob/master/COPYRIGHT>).

The port is actively maintained (commit dated 2026-08-22) and **DOS is still a
live target, but strictly through Borland**. The README: "Turbo Vision can still
be built either as a DOS or Windows library with Borland C++. Obviously, there is
no Unicode support here", confirmed with "Borland C++ 4.52 with the Borland
PowerPack for DOS" and "Turbo Assembler 4.0", and CI publishes
`examples-dos.zip` (16-bit) and `examples-dpmi32.zip`
(<https://github.com/magiblot/tvision>). There is no Open Watcom or DJGPP build
path. The modern CMake path needs C++14 and `libncursesw` and is explicitly not
Borland.

Which means this path depends on Borland C++ 4.52 and TASM 4.0: proprietary,
not purchasable, and with a 16-bit installer that will not run on a 64-bit host.
Rule it out. For a DJGPP Turbo Vision you use the older Sigala/SET port,
`tv210b.zip` in the DJGPP archive, which tvision's own README acknowledges.

### Which stacks can link a TCP/IP library

**mTCP is 16-bit real mode and Open Watcom-oriented.** Its site: "mTCP is
developed using Open Watcom, an open source tool chain that supports C, C++, and
assembler", needs "An IBM PC compatible with an 8088 or better CPU", 96-384 KB of
memory, "DOS 2.1 or newer", and a packet driver; and "Porting to other
environments such as Borland Turbo C++ for DOS is possible without too much
pain" (<http://www.brutman.com/mTCP/>). The repo README is exact: "mTCP is
compiled using Open Watcom 1.9 under Windows, cross compiled for 16 bit DOS",
per-app `MAKEFILE` driven by `wmake` (<https://github.com/mbbrutman/mTCP>). Last
commit 2026-06-30. **It is not a protected-mode library, so DJGPP is out.**

**Watt-32 supports far more compilers.** Its homepage lists "GNU C/C++ 2.7 (or
later) with djgpp 2.x DOS-extender", "Borland C/C++ 4.x (or later),
small/large/flat (PowerPak) models", "Watcom C/C++ 11.x (or later),
small/large/flat (DOS4GW/Pharlap) models", Metaware HighC, Digital Mars and
others, and states "Both 16-bit real-mode and 32-bit protected-mode is
supported", with a packet driver required under DOS
(<https://www.watt-32.net/>). The source corroborates: `src/configur.bat`
dispatches on `clang / mingw32 / mingw64 / borland / cygwin / djgpp / orangec /
pellesc / highc / visualc / watcom`
(<https://github.com/gvanem/Watt-32/blob/master/src/configur.bat>). Active,
latest commit 2026-06-24.

### Memory model

The number, from a toolchain's own docs, the DOS/4GW manual in the Open Watcom
Programmer's Guide: "The basic memory layout of an AT machine consists of 640KB
of DOS memory, 384KB of upper memory, and an undetermined amount of extended
memory. DOS memory and upper memory together compose real memory, the memory
that can be addressed when the processor is running in real mode"
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/cpguide.html>). That
ceiling is shared with DOS, the packet driver and any TSRs. mTCP's own 96-384 KB
figure has to fit inside it.

Protected mode buys a flat space: DOS/4GW gives a "zero-based flat memory model"
where "a near pointer is exactly the same thing as a linear address", with its
bundled VMM capped at 32 MB; CauseWay allows "variable-sized segments up to 4GB"
but still needs "100-150KB ... conventional DOS memory" for the extender
(<https://open-watcom.github.io/open-watcom-v2-wikidocs/cw.html>). CWSDPMI
advertises "high performance access up to 4GB of physical memory using 4MB
pages" (<http://sandmann.dotster.com/cwsdpmi/>). The DPMI 1.0 spec is the
underlying contract (<http://www.delorie.com/djgpp/doc/dpmi/>).

The trade is clean. Real mode costs nothing at runtime and runs on an 8088, but
caps you near 640 KB with segment awkwardness. Protected mode gives a flat
multi-megabyte space at the cost of a runtime DPMI host and a 386 floor.

For this job the 640 KB limit is not a real constraint. The module index is
146 KB, and you never need it all resident: parse it into a fixed-size record
array on disk and page through it. The download is streamed to a file.

### Summary

| Toolchain | TUI library | TCP stack | Memory model |
|---|---|---|---|
| Open Watcom V2, 16-bit DOS | `graph.h` text functions + `conio`; PDCurses buildable. No widgets. | **mTCP** (native) and **Watt-32** | Real mode, ~640 KB |
| Open Watcom V2, 32-bit (DOS/4GW, CauseWay) | Same, plus PDCurses flat build | **Watt-32** only | Flat 32-bit, extender bundled |
| DJGPP (go32v2) | **PDCurses 3.9** and **Turbo Vision 2.1.0**, both prebuilt in the official archive | **Watt-32** only | Flat 32-bit, needs CWSDPMI at runtime, 386 floor |
| Free Pascal go32v2 | **Free Vision**, bundled, full widget set | **none**, `Sockets` not built for DOS | Flat 32-bit via GO32v2 |
| Free Pascal msdos (i8086) | Free Vision, legacy non-Unicode | **none** | Real mode, structures <= 64 KB |
| Borland C++ 4.52 + magiblot/tvision | Turbo Vision, maintained upstream | Watt-32; mTCP with porting | Real mode or 32-bit DPMI |

Two combinations are documented, maintained, and buildable on Linux with no
proprietary software:

1. **Open Watcom V2 + mTCP + a hand-rolled TUI.** 16-bit real mode, no extender,
   no second file to ship. mTCP's native compiler is Open Watcom and Open Watcom
   hosts natively on Linux. Least friction, most hand-written UI code.
2. **DJGPP + Watt-32 + Turbo Vision 2.1.0 or PDCurses.** A real widget toolkit
   already ported, both libraries sitting in the official DJGPP archive. Costs
   you CWSDPMI as a runtime dependency and a 386 floor, neither of which matters
   inside DOSBox.

---

## 4. The existing tool, and where the module list actually lives

This is the most useful thing found, and it settles the hardest design question
before any DOS code is written.

### FRUA-MM

`~/Downloads/fruamm.zip` is **FRUA Module Manager release 3 (19-Aug-2020)** by
Joonas Hirvonen, the author of Gold Box Companion. Announced in the thread the
user cited: <https://forums.goldbox.games/index.php?topic=3954.0>. Its own home
is <http://gbc.zorbus.net>, and the download the thread points at is
`http://gbc.zorbus.net/tmp/fruamm.zip`.

Contents (*verified locally*, `unzip -l ~/Downloads/fruamm.zip`, 15 files):

| File | What it is |
|---|---|
| `FRUA_Module_Manager.exe` | 591 KB `PE32 executable for MS Windows 4.00 (GUI), Intel i386`. Delphi, same family as the other GBC tools |
| `Tools/curl.exe` | 3.5 MB curl, 2019 vintage. Copyright notice in `curl.txt` is Daniel Stenberg's |
| `Tools/7za.exe` | 7-Zip Extra command-line extractor |
| `FRUA_Patches/*.tbl` | `CKIT.EXE` byte patches (`remove_copy_protection.tbl`, `add_win_button.tbl`, `frua_v13c.tbl`) |
| `FRUA_Module_Manager.bat` | Two lines. Launches the GOG Galaxy shortcut for Unlimited Adventures |

So FRUA-MM does not implement HTTP at all. It shells out. From `strings` on the
binary (*verified locally*):

```
"%s" -o "%s" %s          <- the curl.exe invocation
"%s" x "%s" -o"%s" -y    <- the 7za.exe invocation
Data\FRUA_Modules.txt    <- the cached module list
```

The `FRUA_Module_Manager.txt` readme states the mechanism outright: "External
program `Tools\curl.exe` is used for downloading the module list and files.
External program `Tools\7za.exe` is used for extracting the downloaded
zip-archives."

Its install step is: download the module zip, extract it and every nested
archive to a temp folder, copy the files into a `<module_name>.DSN` folder.
Applying hacks means restoring the pristine `DISK1`-`DISK3` and `CKIT.EXE` from
a first-run backup, copying the module's hacked files over them, patching
`CKIT.EXE` from the module's `DIFF.TBL`, and writing the module name into
`START.DAT`.

### The source of truth for the module list

There are exactly two URLs in the binary (*verified locally*, `strings | grep -i http`):

```
http://frua.rosedragon.org/
http://frua.rosedragon.org/modulelist/export.php
```

`export.php` is the answer. It is the **UA File Archive**'s machine-readable
module index, and per the thread it was added by the archive's maintainer
(Steven Brobst) specifically so FRUA-MM could consume it. The thread records
him fixing "unwanted line breaks" and "bad data in the database" in it, and
release 3 exists to "use new list format from archive".

Fetched live and *verified locally* (`curl http://frua.rosedragon.org/modulelist/export.php`,
HTTP 200, 146 KB):

- The payload is a `<pre>` block inside a minimal HTML 4.01 page. Strip the
  wrapper and the body is **653 lines, every one of them exactly 11
  pipe-delimited fields**. No exceptions, checked programmatically.
- Fields are: relative path, filename, title, size KB, author, starting level,
  starting equipment, hacked flag, number of dungeons, date, description.
- First row:
  `pc/modules/0-9/0tocat.zip|0tocat.zip|To Catch a Thief|49|Geoffk|2-4|Average|Not Hacked|5|1993-07-27|Lead your party through the town of Dysteri...`
- Field one is a path relative to `http://frua.rosedragon.org/`, so the download
  URL is a plain string concatenation.
- Descriptions are HTML-entity-escaped (`AD&amp;D`). That is the only text
  processing needed beyond splitting on `|`.

This format is about as friendly to a 16-bit DOS parser as anything on the
modern web: line-oriented, fixed field count, no JSON, no XML, no pagination.

### The transport is plain HTTP, and that is not an accident

*Verified locally*, and this is the single most important fact in this document:

- `http://frua.rosedragon.org/modulelist/export.php` returns **HTTP 200 over
  port 80 with no redirect to HTTPS**.
- HTTPS on that host is **broken**: `curl https://frua.rosedragon.org/...` fails
  with `SSL: certificate subject name 'sni.dreamhost.com' does not match target
  hostname 'frua.rosedragon.org'`. It is a DreamHost shared-SNI vhost with no
  certificate of its own.
- A hand-rolled `HTTP/1.0` request with `Host:` and a fake `User-Agent: mTCP
  HTGET`, written straight to a socket, returns the full 146 KB body. No chunked
  transfer encoding, no compression, no cookies, no auth, no redirect.
- Module downloads work the same way: `http://frua.rosedragon.org/pc/modules/0-9/0tocat.zip`
  returns HTTP 200, `application/zip`, 49397 bytes, a valid zip.

The archive is not a museum piece either. `http://frua.rosedragon.org/Changes.2025.htm`
lists additions through **13 November 2025**, and the "new modules" list covers
submissions from 2023 to 2025.

### Canonical FRUA module archives that exist today

The user could not find one. There is one, and it is the site above.

- **UA File Archive**, <http://frua.rosedragon.org/>, self-described as the "UA
  File Archive", with per-year change logs going back to 1997 ("Changes at
  Giga's UA space"), an upload form, a submission address of
  `frua@rosedragon.org`, and separate PC and Mac module listings (full, hacked,
  classic, new). This is the canonical collection. 653 PC modules per its own
  export.
- **Bulk tarball**: `http://frua.rosedragon.org/pc/modules/pc_modules.zip`,
  linked from <http://gbc.zorbus.net/> as "Huge archive (300+ MB) of FRUA
  modules from the UA File Archive". *Verified locally*: HTTP 200, `ETag`
  length 0x16f1e18e = **384 MB**, `Last-Modified: Thu, 01 Jan 2026`. Useful as a
  one-shot seed for a local mirror.
- **Hack tooling** the readme points at, also plain HTTP:
  <http://frua.rosedragon.org/pc/hacks/uadap.zip>.
- **archive.org has the game, not the modules.** The 0MHz FRUA item
  (<https://archive.org/details/unlimited-adventures-0mhz>) states outright that
  "over 560MB of Mods are not included". Also there:
  <https://archive.org/details/msdos_Unlimited_Adventures_1993> (the game),
  <https://archive.org/details/FRUAV12_ZIP> (the 1.2 patch), and
  <https://archive.org/details/wiki-fruafandomcom> (a WikiTeam dump of the FRUA
  wiki). None of these is a module collection.

---
## 4a. Can DOS actually unpack the payloads?

FRUA-MM leans on `7za.exe` for this, so it is worth checking whether the
archives need anything a DOS unzip cannot do. Four real modules were downloaded
and inspected (*verified locally*):

| Module | Entries | Compression | Nested archives | Non-8.3 names |
|---|---|---|---|---|
| `0tocat.zip` | 23 | deflate | none | 0 |
| `1temple.zip` | 295 | deflate | none | 0 |
| `ageworms.zip` | 520 | deflate + store | none | 1 (`DOCS/Monster Stats.xlsx`) |
| `beowolf.zip` | 188 | deflate + store | none | 3 (a `.jpg`, two long `.txt`) |

Findings that matter:

- **Only method 0 (store) and method 8 (deflate).** No bzip2, no LZMA, no
  encryption. Info-ZIP `UNZIP` for DOS handles both.
- **No nested archives** in this sample, despite FRUA-MM's readme promising to
  extract "all subarchives inside the archive". Some older uploads presumably
  have them; it is not the common case.
- **Every actual game data file is already 8.3** (`8X8D1001.TLB`,
  `GEO005.DAT`, `MONST007.DAT`). The only long names are human-facing extras:
  cover art, a spreadsheet, verbose readmes. A DOS unzip that truncates or
  skips those still produces a playable `.DSN` folder.
- Newer modules already ship a top-level `<NAME>.DSN/` directory, which is
  exactly the layout FRUA wants, and `ageworms.zip` contains a `DIFF.TBL`, the
  `CKIT.EXE` patch table FRUA-MM applies.

So the unpack step is not a blocker. It is either a bundled Info-ZIP `UNZIP.EXE`
shelled out to, or a linked-in inflate, which is a few hundred lines.

---

## 5. Verdict

**It is possible, and more easily than it should be.** Every piece exists, is
maintained, and lines up: the emulator gives the DOS guest real TCP/IP with DHCP
and DNS, the archive it needs to talk to serves plain HTTP with no redirect and
no working HTTPS, the index is 653 lines of pipe-delimited text, the payloads are
plain deflate zips of 8.3 filenames, and the compiler and TCP stack that fit
together best both cross-compile from Linux.

There is no research blocker left. The remaining work is all writing code.

### The architecture to build

Target **DOSBox Staging** with `ne2000 = true` set explicitly in the game's conf,
because the default is `true` at 0.82.2 and `false` at 0.83.0-RC1 and both are on
this machine.

Ship four things into the DOS guest:

1. Crynwr `NE2000.COM` 11.4.3, invoked `NE2000 0x60 3 0x300`, with `pktd11a/b/c.zip`
   sources alongside it to satisfy GPLv1.
2. mTCP's `DHCP.EXE`, run once at startup to get 10.0.2.15 and the resolver.
3. Info-ZIP `UNZIP.EXE`, or a linked-in inflate.
4. The manager itself: **Open Watcom V2, 16-bit large model, linked against
   mTCP**, with the text UI written on `graph.h`'s text functions.

It fetches `http://frua.rosedragon.org/modulelist/export.php`, strips the
`<pre>` wrapper, splits on `|`, and writes a fixed-width record file to disk so
the 640 KB limit never becomes a question. Selecting a row concatenates
`http://frua.rosedragon.org/` with field one, streams the zip to a temp file,
unpacks it into `<NAME>.DSN`, applies `DIFF.TBL` to `CKIT.EXE` if present, and
writes the name into `START.DAT`. That last part is not guesswork: it is exactly
what FRUA-MM documents itself doing, and it is reproducible from its readme.

**Do not build a mirror first.** The archive works today, is actively maintained
through November 2025, and is the community's real source of truth. Mirroring it
adds a server to run and a sync job to maintain in exchange for nothing. Keep the
384 MB `pc_modules.zip` as the fallback if the host ever forces HTTPS, and note
that if that day comes, mTCP's HTGET will not even follow the redirect to tell
you why.

The alternative to the whole DOS-native path is the host-side helper: gbs, in
Rust, fetches and unpacks the module and writes `START.DAT`, then launches
DOSBox. Two days of work instead of two weeks, no packet driver, no licence
obligation, no 640 KB, HTTPS for free, and it survives the archive changing
transport.

### Where the DJGPP option sits

If the TUI matters more than the aesthetic, DJGPP is the better engineering
choice: Turbo Vision 2.1.0 and PDCurses 3.9 are both sitting prebuilt in the
official DJGPP archive, so you get a real widget toolkit instead of hand-rolling
list boxes on `_outtext`. The cost is Watt-32 instead of mTCP, CWSDPMI as a
second file, and a 386 floor, none of which matter inside an emulator. It also
happens to be the only path with a route to TLS if that ever becomes necessary.

Pick Open Watcom + mTCP if the point is a small self-contained real-mode binary.
Pick DJGPP + Watt-32 + Turbo Vision if the point is a good-looking UI.

### The real blockers, such as they are

1. **The archive's transport is one DreamHost configuration change away from
   breaking the whole thing.** Its HTTPS certificate is already wrong
   (`sni.dreamhost.com`); the day someone fixes it and adds a redirect, a
   real-mode client is dead. This is the only genuine risk, and it is outside
   your control.
2. **Long filenames in newer modules.** A handful of files per module are cover
   art and verbose readmes with non-8.3 names. Every actual game file is already
   8.3, so a DOS unpack that truncates or skips them still produces a playable
   module, but it will not be a faithful extraction.
3. **`ICMP` does not work through slirp**, so the obvious smoke test lies. Test
   with a TCP fetch, not `PING`.
4. **You write the widgets** on the Open Watcom path. `graph.h` gives text
   windows, colour and cursor control. It does not give you a scrolling list of
   653 items.

### Does the DOS-native constraint buy anything?

Beyond aesthetics, essentially no, and the aesthetics are the actual point, so
say that plainly rather than dressing it up.

The honest accounting: FRUA-MM already solved this problem in 2020 by shelling
out to `curl.exe` from a Delphi GUI, and a host-side Rust implementation would be
strictly better than either on every functional axis. A DOS-native manager is
slower to write, harder to maintain, capped at 640 KB, dependent on a GPLv1
driver you must redistribute sources for, and permanently one HTTPS redirect away
from bricking.

What it buys is one real thing and one arguable one. The real thing: it runs
*inside* the same DOS session as FRUA, so browsing and installing a module never
leaves the emulator, and `START.DAT` is written by a program that can see the
filesystem FRUA will actually read. No path translation, no guessing where the
GOG install went, no host-side model of the DOS drive layout that can drift. The
arguable one: a text-mode module browser next to a 1993 game engine is a genuinely
nice thing to have made.

That is enough reason to build it as a side project. It is not enough reason to
make it the way gbs manages modules.
