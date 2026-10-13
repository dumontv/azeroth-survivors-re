import importlib.util
from pathlib import Path
import struct
import unittest

spec = importlib.util.spec_from_file_location("inspect_original", Path(__file__).with_name("inspect_original.py"))
inspection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspection)


class InspectionTests(unittest.TestCase):
    def test_rejects_changed_executable_before_decoding_or_writing(self):
        with self.assertRaisesRegex(ValueError, "layout decoding refused"):
            inspection.inspect(b"synthetic unrecognized executable")

    def test_rejects_non_elf_and_truncated_section_table(self):
        with self.assertRaisesRegex(ValueError, "ELF64"):
            inspection.Elf(b"not ELF")
        data = bytearray(64)
        data[:6] = b"\x7fELF\x02\x01"
        struct.pack_into("<H", data, 18, 62)
        struct.pack_into("<Q", data, 40, 64)
        struct.pack_into("<HHH", data, 58, 64, 3, 1)
        with self.assertRaisesRegex(ValueError, "truncated ELF"):
            inspection.Elf(data)

    def test_finds_complete_json_inside_binary_noise(self):
        data = b'\xff\0{\n "game.title": "Synthetic", "example": "value"}\0\xff'
        objects = inspection.language_objects(data)
        self.assertEqual(objects, [(2, {"game.title": "Synthetic", "example": "value"})])

    def test_rejects_partial_or_ambiguous_static_tables(self):
        refs = [{"address": 100, "text": "a"}, {"address": 120, "text": "b"}]
        self.assertEqual(inspection.contiguous_records(refs, {"a", "b"}, 20), [(100, "a"), (120, "b")])
        with self.assertRaisesRegex(ValueError, "found 0"):
            inspection.contiguous_records(refs[:1], {"a", "b"}, 20)
        duplicate = refs + [{"address": 200, "text": "a"}, {"address": 220, "text": "b"}]
        with self.assertRaisesRegex(ValueError, "found 2"):
            inspection.contiguous_records(duplicate, {"a", "b"}, 20)

    def test_refuses_tracked_output_locations(self):
        with self.assertRaisesRegex(ValueError, "out/"):
            inspection.ignored_output(inspection.ROOT / "reference/recovered")


if __name__ == "__main__":
    unittest.main()
