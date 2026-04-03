---
title: "Reverse Engineering the Eniac FlexBar: From Bricked Device to Rust Library"
author: Corey Alexander
date: 2026-04-03
tags:
  - rust
  - reverse-engineering
  - hardware
  - usb
---

<!-- DRAFT: Outline for blog post about reverse engineering the Eniac FlexBar -->

## Outline

### 1. What is the FlexBar?

- Intro to the Eniac FlexBar hardware: ESP32-S3 powered, 2170x60 AMOLED touch strip for keyboards
- The companion app: FlexDesigner (Electron), supports plugins for custom key rendering
- The problem: you need FlexDesigner running to do anything interesting with the display. Without it, it's just a USB keyboard with macro buttons.
- The goal: control this thing directly from Rust. No Electron. No JavaScript. Pure USB.

### 2. Starting Point: We Could Blink, But Not Draw

- We'd already reverse-engineered the connection handshake and the MD5-XOR encryption scheme
- Basic commands worked: haptic clicks, sleep/wake
- But profile uploads (sending actual images to display) crashed the device every time
- Something about our packet format was wrong -- we needed to capture what FlexDesigner actually sends

### 3. First Attempt: Inspector-Style Debugging

- Natural first instinct: Electron app means we can use Chrome DevTools / Node.js inspector
- Launched FlexDesigner with `--inspect=9229` to get debugger access
- Started poking around, monkey-patching things to capture serial traffic
- **But FlexDesigner couldn't connect to the device at all.** It would try to handshake and just... nothing.
- At this point we didn't know why. Was the inspector interfering? Was the device broken? Was our approach fundamentally wrong?
- Assumed the inspector/CDP approach was a dead end -- maybe intercepting at the JavaScript level was too high up the stack
- This is what sent us down the rabbit hole...

### 4. The DYLD Hook Detour (Going Lower)

- If JavaScript-level interception won't work, go lower: intercept raw `write()` syscalls
- Built a DYLD_INTERPOSE hook in C to capture everything going to the serial port
- Had to use raw ARM64 `svc #0x80` syscalls because `dlsym` causes infinite recursion with DYLD_INTERPOSE
- Couldn't use `__thread` TLS either -- crashes Electron. `volatile int` instead.
- It worked! Captured the init+config handshake. Progress!
- But FlexDesigner STILL couldn't connect to the device. Same problem as before.
- Started exploring a Linux VM route too -- maybe this was a macOS/SIP issue?

### 5. The Safe Mode Breakthrough

- Turns out the device was in a bad state from our earlier crash-inducing upload experiments
- Discovery: hold the button during power-on to boot into **safe mode**, which resets the device to a clean state
- **Simple fix. Hours of confusion.**
- Once we knew about safe mode, everything changed: FlexDesigner could connect again
- Crucial realization: the inspector approach wasn't broken -- the *device* was broken. We'd been chasing the wrong problem.

### 6. Back to the Inspector (The Real Breakthrough)

- With the device healthy again, went straight back to Chrome DevTools Protocol
- This time FlexDesigner connected fine with `--inspect=9229` -- confirming the inspector was never the problem
- First hurdle: `require` doesn't exist in Electron 38's inspector eval context -- used `process.mainModule.require.bind(process.mainModule)` instead
- Scanned `Module._cache`: 1053 loaded modules, including the full `@serialport/stream` stack
- Monkey-patched `SerialPortStream.prototype.write` to capture every serial write
- This gave us encrypted packets. But we needed the plaintext too.
- The DYLD detour wasn't wasted -- it taught us about Electron's process architecture and confirmed the hook approach had its own problems (child processes don't inherit DYLD_INSERT_LIBRARIES in Electron 38)

### 7. Cracking the Cipher (Set Intersection Math)

- The encryption scheme: `XOR(plaintext, MD5(plaintext))` -- deterministic, no external key
- Patched `crypto.createHash` to intercept the plaintext before encryption and log it alongside the ciphertext
- The captured data revealed our inner header format was completely wrong:
  - We assumed: version, pack_size, total_transfer_size, chunk_size
  - Actual: chunk_index (0-based), num_chunks, data_size (varies per chunk), offset into payload
- And the payload format was structured: magic header + JSON layout descriptor + concatenated PNGs
- **First upload worked!** Sent a solid gray PNG, device ACKed it. Six single-color uploads in a row, all successful.
- Then multi-key uploads (6+ widgets) failed silently. The cipher header had 8 wrong `ptIdx` values -- they'd been tested against zero-padded regions where `0 XOR anything == anything`
- **Three-capture solve**: captured 3 different uploads, computed candidate plaintext positions for each cipher byte, intersected the sets across captures. All 8 corrections were off by just 1-4 positions.
- **Nine-key rainbow on the bar.** Red, orange, yellow, green, cyan, blue, purple, magenta, white.

### 8. The Direct Draw Dead End (and Finding the Real Path)

- Profile uploads require a 7-second device reboot. Not useful for live updates.
- FlexDesigner SDK advertises `plugin.directDraw()` for 30fps rendering
- Tried every rt=9 (DirectDraw) variant: raw JPEG, encrypted JPEG, JSON+JPEG, various CIDs. Device accepted everything silently and did nothing.
- rt=9 is a ghost. The firmware doesn't implement it.
- **The real path**: sniffed plugin traffic from a real FlexDesigner plugin
  - Built a test plugin, linked it to FlexDesigner, captured all serial traffic while tapping keys
  - Draws use **rt=8 (Special)** -- the same receiver type as the connection handshake
  - Simple format: tiny JSON event + PNG image, no encryption, no chunking
  - But it only works on keys registered as plugin keys (`typeOverride: "plugin"`)
  - Added `.drawable()` to the Rust API -- one method call to register a key for live drawing

### 9. The Payoff: Interactive Touch-to-Color in Pure Rust

- The final demo: 3 drawable keys, tap any key to cycle through 8 colors instantly
- The loop: device sends plugin click event -> Rust generates new PNG (~200 bytes) -> sends draw command -> display updates in sub-millisecond time
- No FlexDesigner. No Electron. No JavaScript. Pure Rust, direct USB serial.
- Show the code: the full API is ~10 lines for connecting, uploading a profile, and running an interactive event loop

### 10. V8 Bytecode Decompilation (Bonus Round)

- FlexDesigner's main process is compiled to V8 bytecode (.jsc files) via bytenode
- Decompiling it required a trick: patch the flag hash (offset 12) AND platform hash (offset 16) in the bytecode file to match the host machine
- This let us load macOS ARM64 bytecode on Linux ARM64 and get `--print-bytecode` to dump the full V8 disassembly (71k lines)
- Confirmed protocol details, discovered additional features we hadn't found through traffic capture alone

### 11. What's Next

- Text rendering: live CPU/memory/disk stats on the bar (sysmonitor demo already working)
- Multi-page profiles with native page navigation
- Daemon architecture: a compositor that accepts key registrations from multiple apps (Mull task approval, CI status, media controls)
- The library is open source: link to coreyja-studio/flexbar-rs

### 12. Demo Section Ideas

<!-- These could be embedded videos, GIFs, or photos -->

- Photo: the FlexBar with a rainbow profile uploaded
- Video/GIF: tapping keys to cycle colors in real time
- Video/GIF: sysmonitor demo showing live CPU/memory stats
- Screenshot: the 10-line Rust code that does all of this
- Side-by-side: FlexDesigner UI vs raw Rust API achieving the same result

---

## Notes for Writing

**Tone**: Adventure/detective story. Each dead end leads to a pivot that teaches something. The narrative arc bends on a misdiagnosis -- we thought the inspector approach was broken when really the device was broken. Classic "chasing the wrong problem."

**Key themes**:
- Reverse engineering as detective work: each failure narrows the search space
- Misdiagnosis: the first approach was right all along, we just didn't know it yet
- The detour wasn't wasted -- DYLD work taught us about Electron's process model, and safe mode discovery unlocked everything
- The power of the right abstraction level (fighting C syscalls vs monkey-patching JavaScript)
- "Off by 1-4 positions" -- how close we were the whole time
- From bricked device to interactive demos

**Target length**: Long-form technical post (2000-3000 words). Enough code to be instructive, enough narrative to be entertaining.

**Working title alternatives**:
- "Reverse Engineering the Eniac FlexBar: From Bricked Device to Rust Library"
- "How We Reverse Engineered a USB Touch Bar in One Session"
- "DYLD Hooks, Chrome DevTools, and Set Intersection: Reverse Engineering the FlexBar"
