#!/usr/bin/env python3
"""Ask the terminal, in its own words, how big an image it will accept.

Transmits solid-colour PNGs of known pixel dimensions over the kitty graphics
protocol with `q=0` (replies ON) and prints what comes back -- OK, or the error
the real pipeline never sees because it runs `q=2`.  Each image is deleted
immediately, so every probe is independent and nothing accumulates against a
storage quota.  Nothing is displayed: `a=t` stores without placing.

Run it IN the terminal you care about -- it needs the real tty for TIOCGWINSZ
and for the protocol replies, so a pipe or a harness will not do.  Terminal
state is restored on any exit.

SL-245 acceptance evidence.  What it established, now recorded in
`mem.fact.kitty-graphics.image-size-limit`: the bound is per-side (10000 px in
both ghostty and kitty), NOT area and not PNG bytes -- 8000x4000 stores while
2000x16000, at the same area, does not.  And ghostty answers a refused image
with nothing at all, even at `q=0`, which is why `-X` must refuse before it
writes rather than handle an error after.

Read "no reply within the timeout" as a refusal, not as a bug in this script.
"""
import fcntl, os, select, struct, sys, termios, tty, zlib

CHUNK = 4096
TIMEOUT = 3.0


def window():
    rows, cols, xpx, ypx = struct.unpack(
        "HHHH", fcntl.ioctl(sys.stdout, termios.TIOCGWINSZ, b"\0" * 8)
    )
    return rows, cols, xpx, ypx


def png(width, height):
    """A solid-colour PNG: kilobytes on the wire, `width * height` decoded."""
    def chunk(tag, data):
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    deflate = zlib.compressobj(1)
    row = b"\x00" + b"\x40\x80\xc0" * width
    body = bytearray()
    for _ in range(height):
        body += deflate.compress(row)
    body += deflate.flush()
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", bytes(body))
        + chunk(b"IEND", b"")
    )


def send(payload, image_id):
    import base64

    encoded = base64.standard_b64encode(payload)
    first = True
    while encoded:
        piece, encoded = encoded[:CHUNK], encoded[CHUNK:]
        keys = f"i={image_id},q=0,a=t,f=100,t=d," if first else ""
        os.write(1, b"\x1b_G" + f"{keys}m={1 if encoded else 0}".encode() + b";" + piece + b"\x1b\\")
        first = False


def reply():
    out = b""
    while select.select([sys.stdin], [], [], TIMEOUT)[0]:
        out += os.read(sys.stdin.fileno(), 4096)
        if out.endswith(b"\x1b\\"):
            break
    if not out:
        return "(no reply)"
    body = out.split(b";", 1)[-1].replace(b"\x1b\\", b"")
    return body.decode("utf-8", "replace").strip() or "(empty)"


def main():
    rows, cols, xpx, ypx = window()
    print(f"window: {cols}x{rows} cells, {xpx}x{ypx} px")
    if not xpx:
        sys.exit("terminal reports no pixel size")

    saved = termios.tcgetattr(sys.stdin)
    try:
        tty.setraw(sys.stdin.fileno())
        image_id = 100
        # Fixed width (the window's), rising height -- the shape `fit_box` makes.
        for megapixels in (8, 16, 24, 32, 40, 48, 64, 80):
            height = (megapixels * 1_000_000) // xpx
            image_id += 1
            send(png(xpx, height), image_id)
            verdict = reply()
            os.write(1, f"  {xpx:5d} x {height:6d}  = {megapixels:3d} Mpx  ->  {verdict}\r\n".encode())
            os.write(1, b"\x1b_Ga=d,d=I,i=" + str(image_id).encode() + b"\x1b\\")

        # Same areas, different shapes -- is the bound on area or on a side?
        os.write(1, b"\r\n  same 32 Mpx, different shapes:\r\n")
        for width, height in ((2000, 16000), (8000, 4000), (32000, 1000), (1000, 32000)):
            image_id += 1
            send(png(width, height), image_id)
            verdict = reply()
            os.write(1, f"  {width:5d} x {height:6d}  = {width*height//1000000:3d} Mpx  ->  {verdict}\r\n".encode())
            os.write(1, b"\x1b_Ga=d,d=I,i=" + str(image_id).encode() + b"\x1b\\")
    finally:
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, saved)


main()
