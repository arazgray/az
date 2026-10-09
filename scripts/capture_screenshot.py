#!/usr/bin/env python3
"""Capture az's real welcome screen as the README PNG (POSIX + Pillow)."""

from __future__ import annotations

import argparse
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time
import unicodedata
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as error:
    raise SystemExit("Install Pillow to capture screenshots: python3 -m pip install Pillow") from error


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_FG = (192, 202, 245)
DEFAULT_BG = (26, 27, 38)


def drain(master: int, output: bytearray, seconds: float) -> None:
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        timeout = min(0.05, max(0.0, deadline - time.monotonic()))
        ready, _, _ = select.select([master], [], [], timeout)
        if not ready:
            continue
        try:
            chunk = os.read(master, 65536)
        except OSError:
            return
        if not chunk:
            return
        output.extend(chunk)


def render_terminal(output: bytes, cols: int, rows: int) -> list[list[tuple[str, tuple[int, int, int], tuple[int, int, int], bool, bool]]]:
    grid = [[(" ", DEFAULT_FG, DEFAULT_BG, False, False) for _ in range(cols)] for _ in range(rows)]
    row, col = 1, 1
    fg, bg = DEFAULT_FG, DEFAULT_BG
    bold = underline = False
    text = output.decode("utf-8", "replace")

    def sgr(params: str) -> None:
        nonlocal fg, bg, bold, underline
        nums = [int(part) if part.isdigit() else 0 for part in params.split(";") if part]
        if not nums:
            nums = [0]
        i = 0
        while i < len(nums):
            code = nums[i]
            if code == 0:
                fg, bg, bold, underline = DEFAULT_FG, DEFAULT_BG, False, False
            elif code == 1:
                bold = True
            elif code in (21, 22):
                bold = False
            elif code == 4:
                underline = True
            elif code == 24:
                underline = False
            elif code == 39:
                fg = DEFAULT_FG
            elif code == 49:
                bg = DEFAULT_BG
            elif code in (38, 48) and i + 4 < len(nums) and nums[i + 1] == 2:
                color = tuple(nums[i + 2 : i + 5])
                if code == 38:
                    fg = color
                else:
                    bg = color
                i += 4
            i += 1

    i = 0
    while i < len(text):
        char = text[i]
        if char == "\x1b":
            if i + 1 < len(text) and text[i + 1] == "[":
                end = i + 2
                while end < len(text) and not ("@" <= text[end] <= "~"):
                    end += 1
                if end >= len(text):
                    break
                params, final = text[i + 2 : end], text[end]
                if final == "m":
                    sgr(params)
                elif final == "H":
                    values = [int(part) if part.isdigit() else 1 for part in params.split(";")]
                    row = values[0] if values and values[0] else 1
                    col = values[1] if len(values) > 1 and values[1] else 1
                elif final == "J" and params in ("", "2"):
                    grid = [[(" ", DEFAULT_FG, DEFAULT_BG, False, False) for _ in range(cols)] for _ in range(rows)]
                    if params in ("", "2"):
                        row, col = 1, 1
                elif final == "K" and 1 <= row <= rows:
                    for x in range(max(0, col - 1), cols):
                        grid[row - 1][x] = (" ", fg, bg, bold, underline)
                i = end + 1
                continue
            if i + 1 < len(text) and text[i + 1] == "]":
                end = i + 2
                while end < len(text) and text[end] not in ("\x07", "\x1b"):
                    end += 1
                i = min(len(text), end + (1 if end < len(text) and text[end] == "\x07" else 2))
                continue
            i += 2
            continue
        if char == "\r":
            col = 1
        elif char == "\n":
            row = min(rows, row + 1)
        elif char == "\t":
            col = min(cols, ((col - 1) // 4 + 1) * 4 + 1)
        elif ord(char) >= 32 and ord(char) != 127 and not unicodedata.combining(char):
            if char not in "\u200e\u2066\u2067\u2068\u2069":
                width = 2 if unicodedata.east_asian_width(char) in ("W", "F") else 1
                if 1 <= row <= rows and 1 <= col <= cols:
                    grid[row - 1][col - 1] = (char, fg, bg, bold, underline)
                    for extra in range(1, width):
                        if col + extra <= cols:
                            grid[row - 1][col + extra - 1] = ("", fg, bg, False, False)
                col = min(cols, col + width)
        i += 1
    return grid


def font_or_default(path: str, size: int):
    try:
        return ImageFont.truetype(path, size)
    except OSError:
        return ImageFont.load_default()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", default="README.md", help="file to show behind the welcome dialog")
    parser.add_argument("--output", default=str(ROOT / "screenshot.png"), help="destination PNG")
    parser.add_argument("--cols", type=int, default=168)
    parser.add_argument("--rows", type=int, default=45)
    args = parser.parse_args()
    if os.name != "posix":
        raise SystemExit("Screenshot capture requires a POSIX pseudo-terminal.")
    if args.cols < 30 or args.rows < 10:
        raise SystemExit("Use at least 30 columns and 10 rows.")

    subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)
    target_path = Path(args.file).expanduser()
    target = (target_path if target_path.is_absolute() else ROOT / target_path).resolve()
    binary = ROOT / "target" / "release" / "az"
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", args.rows, args.cols, 0, 0))
    output = bytearray()
    process = None
    with tempfile.TemporaryDirectory(prefix="az-screenshot-") as state_dir:
        env = os.environ.copy()
        env.update({
            "TERM": "xterm-256color",
            "COLUMNS": str(args.cols),
            "LINES": str(args.rows),
            "AZ_NO_UPDATE_CHECK": "1",
            "XDG_STATE_HOME": state_dir,
        })
        process = subprocess.Popen(
            [str(binary), str(target)],
            cwd=ROOT,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            start_new_session=True,
        )
        os.close(slave)
        try:
            drain(master, output, 0.8)
            os.write(master, b"\x1bw")  # Alt+W opens the current welcome screen.
            drain(master, output, 0.5)
            captured = bytes(output)
        finally:
            try:
                os.write(master, b"\r\x11")  # Dismiss, then quit the clean buffer.
                drain(master, output, 0.3)
            except OSError:
                pass
            try:
                process.wait(timeout=1.0)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=1.0)
            os.close(master)

    if b"Welcome" not in captured:
        raise SystemExit("Did not capture the welcome dialog; check the terminal run output.")
    grid = render_terminal(captured, args.cols, args.rows)
    cell_w, cell_h = 10, 20
    image = Image.new("RGB", (args.cols * cell_w, args.rows * cell_h), DEFAULT_BG)
    draw = ImageDraw.Draw(image)
    regular = font_or_default("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 15)
    bold_font = font_or_default("/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 15)
    for y, cells in enumerate(grid):
        for x, (char, cell_fg, cell_bg, cell_bold, cell_underline) in enumerate(cells):
            x0, y0 = x * cell_w, y * cell_h
            draw.rectangle((x0, y0, x0 + cell_w - 1, y0 + cell_h - 1), fill=cell_bg)
            if char:
                draw.text((x0 + 1, y0 + 1), char, font=bold_font if cell_bold else regular, fill=cell_fg)
                if cell_underline:
                    draw.line((x0, y0 + cell_h - 3, x0 + cell_w - 1, y0 + cell_h - 3), fill=cell_fg)
    output_path = Path(args.output).expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(output_path, optimize=True)
    print(f"Wrote {output_path} ({args.cols}x{args.rows} terminal cells)")


if __name__ == "__main__":
    main()
