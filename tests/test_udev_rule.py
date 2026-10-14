"""Keep the Mini input permission rule narrowly scoped and seat-based."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RULE = ROOT / "udev" / "71-apex-pro-mini-input.rules"


class UdevRuleTests(unittest.TestCase):
    def test_minimally_scoped_uaccess_acl(self):
        noncomments = [
            line.strip()
            for line in RULE.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        ]
        self.assertEqual(len(noncomments), 1, "Do not add broad ACL rules")
        line = noncomments[0]
        for constraint in (
            'ACTION!="remove"',
            'SUBSYSTEM=="input"',
            'KERNEL=="event*"',
            'ATTRS{idVendor}=="1038"',
            'ATTRS{idProduct}=="161e"',
            'ENV{ID_INPUT_KEYBOARD}=="1"',
            'MODE="0660"',
            'TAG+="uaccess"',
        ):
            with self.subTest(constraint=constraint):
                self.assertIn(constraint, line)
        self.assertNotIn('MODE="0666"', line)
        self.assertNotIn('GROUP="input"', line)

    def test_loads_before_systemd_seat_late(self):
        self.assertLess(int(RULE.name.split("-")[0]), 73)


if __name__ == "__main__":
    unittest.main()
