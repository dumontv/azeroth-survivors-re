#!/usr/bin/env python3
"""Read-only inspection of a known original ELF; recovered content stays in ignored output.

Never executes the input. Layout decoding is pinned to a SHA-256 rather than trusting a version
string. A changed binary requires a newly investigated layout, not a force flag.
"""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess


KNOWN_SHA256 = "a87fef2a369bb37ec26b3cad8b3834f92f969d3c9ecc8bd98ffcaab2927ee134"
ROOT = Path(__file__).resolve().parents[3]


class Elf:
    """The ELF64 sections and relative relocations needed to read Rust static strings."""

    def __init__(self, data):
        self.data = data
        if len(data) < 64 or data[:6] != b"\x7fELF\x02\x01":
            raise ValueError("expected a little-endian ELF64 file")
        if self.unpack("<H", 18)[0] != 62:
            raise ValueError("expected the x86-64 original executable")
        table = self.unpack("<Q", 40)[0]
        stride, count, names_index = self.unpack("<HHH", 58)
        if stride != 64 or not 0 < names_index < count:
            raise ValueError("unsupported ELF section table")
        self.sections = [self.unpack("<IIQQQQIIQQ", table + i * stride) for i in range(count)]
        names = self.sections[names_index]
        names = self.slice(names[4], names[5])
        self.named = {}
        for section in self.sections:
            name = names[section[0]:].split(b"\0", 1)[0].decode("ascii")
            self.named[name] = section
        for name in (".rodata", ".rela.dyn", ".data.rel.ro"):
            if name not in self.named:
                raise ValueError(f"missing ELF section {name}")
        self.relocations = {}
        section = self.named[".rela.dyn"]
        if section[9] != 24 or section[5] % 24:
            raise ValueError("unsupported ELF relocation table")
        for pos in range(section[4], section[4] + section[5], 24):
            address, info, target = self.unpack("<QQq", pos)
            if info == 8:  # R_X86_64_RELATIVE, no symbol
                self.relocations[address] = target

    def slice(self, pos, length):
        if pos < 0 or length < 0 or pos + length > len(self.data):
            raise ValueError("truncated ELF data")
        return self.data[pos:pos + length]

    def unpack(self, fmt, pos):
        return struct.unpack(fmt, self.slice(pos, struct.calcsize(fmt)))

    def offset(self, address):
        for section in self.sections:
            if section[1] != 8 and section[3] <= address < section[3] + section[5]:
                return section[4] + address - section[3]
        raise ValueError(f"unmapped ELF address {address:x}")

    def value(self, address, fmt):
        return self.unpack(fmt, self.offset(address))[0]

    def string(self, address):
        target = self.relocations.get(address)
        if target is None:
            raise ValueError(f"missing string relocation at {address:x}")
        length = self.value(address + 8, "<Q")
        if not 0 < length <= 4096:
            raise ValueError("invalid static string length")
        return self.slice(self.offset(target), length).decode("utf-8")

    def string_references(self):
        ro = self.named[".rodata"]
        result = []
        for address, target in self.relocations.items():
            if not ro[3] <= target < ro[3] + ro[5]:
                continue
            try:
                text = self.string(address)
                if any(ord(c) < 32 and c not in "\r\n\t" for c in text):
                    continue
            except (ValueError, UnicodeError):
                continue
            result.append({"address": address, "text": text})
        return result


def language_objects(data):
    """Find standalone UTF-8 JSON objects, retaining their byte offsets as evidence."""
    decoder = json.JSONDecoder()
    result = []
    pattern = rb'\{\s*\r?\n\s*"[^"\n]{1,100}"\s*:'
    for match in re.finditer(pattern, data):
        pos = match.start()
        candidate = data[pos:pos + 4_000_000].decode("utf-8", "replace")
        try:
            obj, end = decoder.raw_decode(candidate)
        except (ValueError, RecursionError):
            continue
        if not isinstance(obj, dict) or "game.title" not in obj:
            continue
        original = candidate[:end].encode("utf-8")
        if data[pos:pos + len(original)] != original:
            raise ValueError("language object is not intact UTF-8")
        if not all(isinstance(k, str) and isinstance(v, str) for k, v in obj.items()):
            raise ValueError("language object has unexpected value types")
        result.append((pos, obj))
    return result


def contiguous_records(refs, keys, stride, string_at=0):
    """Find one complete static table; ambiguous or partial matches fail closed."""
    matches = {r["address"]: r["text"] for r in refs if r["text"] in keys}
    candidates = []
    for address in sorted(matches):
        addresses = [address + i * stride for i in range(len(keys))]
        values = [matches.get(a) for a in addresses]
        if set(values) == keys:
            candidates.append([(a - string_at, matches[a]) for a in addresses])
    if len(candidates) != 1:
        raise ValueError(f"expected one complete {len(keys)}-record table, found {len(candidates)}")
    return candidates[0]


def inspect(data):
    digest = hashlib.sha256(data).hexdigest()
    if digest != KNOWN_SHA256:
        raise ValueError(f"unrecognized executable SHA-256 {digest}; layout decoding refused")
    elf = Elf(data)
    languages = language_objects(data)
    english = [obj for _, obj in languages if obj.get("common.female") == "Female"]
    if len(english) != 1:
        raise ValueError("expected exactly one English language table")
    english = english[0]
    refs = elf.string_references()
    keys = lambda prefix: {k[len(prefix):-5] for k in english if k.startswith(prefix) and k.endswith(".name")}
    class_rows = contiguous_records(refs, keys("class."), 144)
    classes = []
    for address, key in class_rows:
        row = {
            "key": key, "id": elf.value(address + 124, "<B"),
            "health": elf.value(address + 112, "<f"),
            "start": elf.string(address + 48), "address": hex(address),
        }
        if elf.string(address + 16) != english[f"class.{key}.name"]:
            raise ValueError("class display name disagrees with the language table")
        if row["id"] not in (1, 2, 3, 4, 5, 7, 8, 9, 11) or not 1 < row["health"] < 10_000:
            raise ValueError("class record failed layout sanity checks")
        classes.append(row)
    abilities = []
    for address, key in contiguous_records(refs, keys("ability."), 120):
        if elf.string(address + 16) != english[f"ability.{key}.name"]:
            raise ValueError("ability display name disagrees with the language table")
        # Keep raw offsets for fields whose combat semantics are not yet traced.
        abilities.append({
            "key": key, "name": elf.string(address + 16), "icon": elf.string(address + 32),
            "address": hex(address), "spell_id": elf.value(address + 64, "<I"),
            "raw_u32_48_116": [elf.value(address + i, "<I") for i in range(48, 120, 4)],
            "raw_f32_68_112": [elf.value(address + i, "<f") for i in range(68, 116, 4)],
        })
    zones = []
    for address, key in contiguous_records(refs, keys("zone."), 184):
        if elf.string(address + 16) != english[f"zone.{key}.name"]:
            raise ValueError("zone display name disagrees with the language table")
        zones.append({"key": key, "name": elf.string(address + 16), "address": hex(address)})
    if len({c["id"] for c in classes}) != 9 or any(c["start"] not in keys("ability.") for c in classes):
        raise ValueError("class identity or starting ability validation failed")
    paths = sorted({s.decode() for s in re.findall(rb'crates[\\/][A-Za-z0-9_.\\/-]{1,240}\.rs', data)})
    report = {
        "source_sha256": digest, "source_size": len(data),
        "language_tables": [{"offset": hex(pos), "entries": len(obj)} for pos, obj in languages],
        "original_game_modules": [p for p in paths if "wenilla-rogue" in p],
        "classes": classes, "abilities": abilities, "zones": zones,
        "language_key_groups": dict(Counter(k.split(".")[0] for k in english)),
        "limits": [
            "Static records and text are evidence of content, not recovered source code.",
            "Only starting spells and class health are applied by the reference profile.",
            "Other ability fields retain offsets and raw values until their semantics are traced.",
            "Talents, loot, progression, enemies, combat balance and screens still need reconstruction.",
        ],
    }
    return report, languages


def profile_text(report):
    lines = ["schema = 1", f'source_sha256 = "{report["source_sha256"]}"']
    for row in report["classes"]:
        lines += ["", "[[classes]]", f'id = {row["id"]}',
                  f'health = {row["health"]}', f'start = {json.dumps(row["start"])}']
        if row["start"] == "auto_shot":
            ability = next(a for a in report["abilities"] if a["key"] == "auto_shot")
            raw = ability["raw_f32_68_112"]
            lines += ["[classes.projectile]", f'spell = {ability["spell_id"]}',
                      f'damage = {raw[0]}', f'cooldown = {raw[1]}', f'range = {raw[2]}',
                      f'count = {ability["raw_u32_48_116"][9]}']
    return "\n".join(lines) + "\n"


def ignored_output(path):
    path = path.resolve()
    if not path.is_relative_to(ROOT):
        raise ValueError("output must be inside this checkout's ignored out/ directory")
    if not path.is_relative_to(ROOT / "out"):
        raise ValueError("recovered content may only be written under out/")
    check = subprocess.run(["git", "check-ignore", "-q", str(path / "probe")], cwd=ROOT)
    if check.returncode:
        raise ValueError("output is not ignored by Git")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "out/original-analysis")
    args = parser.parse_args()
    try:
        output = ignored_output(args.output)
        report, languages = inspect(args.executable.read_bytes())
        output.mkdir(parents=True, exist_ok=True)
        for pos, language in languages:
            (output / f"language-{pos:x}.json").write_text(
                json.dumps(language, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        (output / "evidence.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        (output / "survivors-reference.toml").write_text(profile_text(report), encoding="utf-8")
    except (OSError, ValueError, struct.error) as error:
        parser.exit(1, f"inspection failed: {error}\n")
    print(f"Source SHA-256: {report['source_sha256']}")
    print(f"Recovered {len(languages)} language tables, {len(report['classes'])} classes, "
          f"{len(report['abilities'])} abilities, {len(report['zones'])} zones")
    for row in report["classes"]:
        print(f"  {row['key']:8} health={row['health']:g} start={row['start']}")
    print(f"Local evidence and profile: {output}")


if __name__ == "__main__":
    main()
