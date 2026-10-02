#!/usr/bin/env python3
"""生成 AkiHuishi 应用图标:深色圆角底 + 橙色双尖叶(秋叶)+ 茎。
2048 超采样渲染,1024 输出,纯 Python 写 PNG(不依赖 Pillow)。"""
import zlib
import struct
import math

S = 2048
OUT = 1024


def inside_bg(x, y):
    dx = abs(x - S / 2)
    dy = abs(y - S / 2)
    hw = S / 2 - 8
    rad = 0.42 * S
    if dx > hw or dy > hw:
        return False
    if dx <= hw - rad or dy <= hw - rad:
        return True
    cx, cy = hw - rad, hw - rad
    return (dx - cx) ** 2 + (dy - cy) ** 2 <= rad ** 2


def inside_leaf(x, y):
    dx = (x - S / 2) / (S / 2)
    dy = (y - S / 2) / (S / 2)
    t = math.atan2(dy, dx) - math.pi / 2
    r = math.hypot(dx, dy)
    return r <= 0.44 * (abs(math.sin(t)) ** 0.8)


def inside_stem(x, y):
    dx = (x - S / 2) / (S / 2)
    dy = (y - S / 2) / (S / 2)
    if 0.28 <= dy <= 0.52:
        cx = 0.03 + 0.12 * (dy - 0.28)
        return abs(dx - cx) <= 0.026
    return False


def leaf_color(t):
    t = max(0.0, min(1.0, t))
    return (
        int(252 + (226 - 252) * t),
        int(186 + (122 - 186) * t),
        int(110 + (44 - 110) * t),
    )


def render():
    rows = []
    factor = S // OUT
    for oy in range(OUT):
        row = bytearray(b"\x00" * (OUT * 4))
        for ox in range(OUT):
            r_ = g_ = b_ = a_ = 0
            for sy in (0, 1):
                for sx in (0, 1):
                    x = ox * factor + sx * (factor // 2)
                    y = oy * factor + sy * (factor // 2)
                    if inside_bg(x, y):
                        if inside_leaf(x, y):
                            c = leaf_color((y / S - 0.26) / 0.52) + (255,)
                        elif inside_stem(x, y):
                            c = (196, 90, 32, 255)
                        else:
                            g = int(26 - 10 * (y / S))
                            c = (g + 2, g + 3, g + 8, 255)
                    else:
                        c = (0, 0, 0, 0)
                    a = c[3]
                    r_ += c[0] * a
                    g_ += c[1] * a
                    b_ += c[2] * a
                    a_ += a
            i = ox * 4
            if a_ == 0:
                continue
            row[i] = r_ // a_
            row[i + 1] = g_ // a_
            row[i + 2] = b_ // a_
            row[i + 3] = a_ // 4
        rows.append(bytes(row))
    return rows


def write_png(path, rows):
    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", OUT, OUT, 8, 6, 0, 0, 0)
    body = zlib.compress(b"".join(b"\x00" + r for r in rows), 9)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", body)
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as f:
        f.write(png)


if __name__ == "__main__":
    rows = render()
    out = "assets/icon.png"
    write_png(out, rows)
    print(f"OK -> {out} ({OUT}x{OUT})")
