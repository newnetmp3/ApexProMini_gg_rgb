#!/usr/bin/env python3
"""Read-only HID report inventory for wired SteelSeries Apex Pro Mini (1038:161e)."""

from pathlib import Path

VENDOR_ID = 0x1038
PRODUCT_ID = 0x161E
CONTROL_USAGE_PAGE = 0xFFC0


def parse_descriptor(raw: bytes):
    """Return (has_0xffc0_usage_page, {(report_id, kind): payload_bits})."""
    pos = 0
    size_bits = count = report_id = usage_page = 0
    saved_globals = []
    reports = {}
    control_usage_seen = False

    while pos < len(raw):
        prefix = raw[pos]
        pos += 1

        if prefix == 0xFE:  # HID long item
            if pos + 2 > len(raw):
                raise ValueError("truncated HID long item header")
            length = raw[pos]
            pos += 2
            if pos + length > len(raw):
                raise ValueError("truncated HID long item payload")
            pos += length
            continue

        length = (0, 1, 2, 4)[prefix & 0x03]
        if pos + length > len(raw):
            raise ValueError("truncated HID short item")
        value = int.from_bytes(raw[pos:pos + length], "little")
        pos += length
        item_type, tag = (prefix >> 2) & 3, prefix >> 4

        if item_type == 1:  # Global items
            if tag == 0:
                usage_page = value
                control_usage_seen |= usage_page == CONTROL_USAGE_PAGE
            elif tag == 7:
                size_bits = value
            elif tag == 8:
                report_id = value
            elif tag == 9:
                count = value
            elif tag == 10:  # Push
                saved_globals.append((usage_page, size_bits, count, report_id))
            elif tag == 11 and saved_globals:  # Pop
                usage_page, size_bits, count, report_id = saved_globals.pop()

        elif item_type == 0 and tag in (9, 11):  # Output / Feature main items
            kind = "Output" if tag == 9 else "Feature"
            key = (report_id, kind)
            reports[key] = reports.get(key, 0) + size_bits * count

    return control_usage_seen, reports


def matches_target(uevent: str) -> bool:
    for line in uevent.splitlines():
        if not line.startswith("HID_ID="):
            continue
        try:
            _bus, vendor, product = line.split("=", 1)[1].split(":")
            if int(vendor, 16) == VENDOR_ID and int(product, 16) == PRODUCT_ID:
                return True
        except ValueError:
            continue
    return False


def run_probe(sysfs_root=Path("/sys/class/hidraw")) -> int:
    found = False
    for node in sorted(sysfs_root.glob("hidraw*")):
        try:
            if not matches_target((node / "device/uevent").read_text()):
                continue
            found = True
            raw = (node / "device/report_descriptor").read_bytes()
            control, reports = parse_descriptor(raw)
            print(f"\n/dev/{node.name} control_page_0xffc0={control}")
            for (report_id, kind), bits in sorted(reports.items()):
                size = (bits + 7) // 8 + 1  # hidraw's report-ID byte
                print(f"  {kind:7} report_id={report_id:3} size={size} bytes")
        except (OSError, ValueError) as exc:
            print(f"{node.name}: {exc}")
    if not found:
        print("No wired Apex Pro Mini (1038:161e) HID interfaces found")
        return 1
    print("\nRead-only: no commands sent to the device.")
    return 0


if __name__ == "__main__":
    raise SystemExit(run_probe())
