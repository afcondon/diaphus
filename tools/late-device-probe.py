#!/usr/bin/env python3
# Regression test for link-spike seeing MIDI devices that appear (or reappear) after it
# started. Needs python-rtmidi (e.g. music/live-coding/midi-oracle/.venv) and a running
# link-spike. Run it twice with the same name: both runs must report 3 of 3.
# A MIDI destination that appears AFTER link-spike started: does link-spike reach it?
import rtmidi, socket, struct, time, sys
name = sys.argv[1] if len(sys.argv) > 1 else "LS-Probe"
def pad(b): return b + b"\0" * (4 - len(b) % 4)
def osc(addr, tags, *args):
    out = pad(addr.encode()) + pad(("," + tags).encode())
    for t, a in zip(tags, args):
        out += pad(a.encode()) if t == "s" else struct.pack(">i", a) if t == "i" else struct.pack(">q", a)
    return out
got = []
mi = rtmidi.MidiIn()
mi.open_virtual_port(name)
mi.set_callback(lambda m, _: got.append(m[0]))
time.sleep(1.0)
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
for k in range(3):
    at = int(time.time() * 1e6) + 50_000
    sock.sendto(osc("/midi/note/at", "siiiih", name, 10, 60 + k, 100, 20, at), ("127.0.0.1", 57122))
    time.sleep(0.3)
time.sleep(0.5)
print(f"{name}: received {sum(1 for m in got if m[0] & 0xF0 == 0x90)} of 3 note-ons")
