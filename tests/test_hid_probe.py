"""Synthetic-only tests: no physical device, no HID writes."""

import unittest
from tools.hid_probe import matches_target, parse_descriptor


class DescriptorTests(unittest.TestCase):
    def test_mini_control_and_report_sizes(self):
        desc = bytes.fromhex(
            "06 c0 ff"      # vendor usage page
            "75 08"         # report size 8 bits
            "96 00 02"      # report count 512
            "b1 02"         # feature report
            "95 40"         # report count 64
            "91 02"         # output report
        )
        vendor, reports = parse_descriptor(desc)
        self.assertTrue(vendor)
        self.assertEqual((reports[(0, "Feature")] + 7) // 8 + 1, 513)
        self.assertEqual((reports[(0, "Output")] + 7) // 8 + 1, 65)

    def test_report_id(self):
        desc = bytes.fromhex("85 07 75 08 95 04 91 02")
        vendor, reports = parse_descriptor(desc)
        self.assertFalse(vendor)
        self.assertEqual(reports[(7, "Output")], 32)

    def test_truncated_hid_item_rejected(self):
        with self.assertRaises(ValueError):
            parse_descriptor(bytes.fromhex("96 ff"))

    def test_target_usb_id(self):
        self.assertTrue(matches_target("HID_ID=0003:00001038:0000161E"))
        self.assertFalse(matches_target("HID_ID=0003:00001038:00001628"))
        self.assertFalse(matches_target("HID_ID=0003:00001234:0000161E"))


if __name__ == "__main__":
    unittest.main()
