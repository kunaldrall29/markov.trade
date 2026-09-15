#!/usr/bin/env python3
"""Assemble an Anchor IDL from the `idl-build` test harness output.

`anchor idl build` runs `cargo test --features idl-build __anchor_private_print_idl`
and stitches the printed blocks together; this does the same stitch without the
CLI, so an IDL snapshot can be refreshed on a box that only has cargo.

    cargo test -p <crate> --features idl-build --lib __anchor_private_print_idl \
      -- --show-output --nocapture --test-threads=1 > out.txt
    python3 scripts/assemble-idl.py out.txt > docs/idl/<name>.json
"""
import json
import re
import sys

text = open(sys.argv[1], encoding="utf8").read()
blocks = re.findall(r"--- IDL begin (\w+) ---\n(.*?)\n--- IDL end \1 ---", text, flags=re.S)
program = None
address = None
errors = []
events = []
extra_types = []
for kind, body in blocks:
    if kind == "program":
        program = json.loads(body)
    elif kind == "address":
        address = json.loads(json.loads(body))
    elif kind == "errors":
        errors.extend(json.loads(body))
    elif kind == "event":
        b = json.loads(body)
        events.append(b["event"])
        extra_types.extend(b.get("types", []))
    elif kind == "constant":
        program_consts = json.loads(body)
if program is None:
    sys.exit("no program block found")
program["address"] = address or program.get("address", "")
if errors:
    program["errors"] = sorted(errors, key=lambda e: e["code"])
if events:
    known = {e["name"] for e in program.get("events", [])}
    program["events"] = sorted(program.get("events", []) + [e for e in events if e["name"] not in known], key=lambda e: e["name"])
    names = {t["name"] for t in program.get("types", [])}
    for t in extra_types:
        if t["name"] not in names:
            program.setdefault("types", []).append(t)
            names.add(t["name"])
    program["types"] = sorted(program["types"], key=lambda t: t["name"])

# The harness prints fully-qualified Rust paths (`crate::module::Type`); the
# CLI shortens them to the last segment everywhere a name or a `defined`
# reference appears, so the snapshot matches what `anchor build` writes.
def shorten(name):
    return name.split("::")[-1] if isinstance(name, str) else name

def walk(node):
    if isinstance(node, dict):
        out = {}
        for k, v in node.items():
            if k == "name" and isinstance(v, str):
                out[k] = shorten(v)
            elif k == "defined" and isinstance(v, dict) and "name" in v:
                out[k] = {**v, "name": shorten(v["name"])}
            else:
                out[k] = walk(v)
        return out
    if isinstance(node, list):
        return [walk(x) for x in node]
    return node

program = walk(program)
for key in ("instructions", "accounts", "events", "types"):
    if key in program:
        program[key] = sorted(program[key], key=lambda x: x["name"])
print(json.dumps(program, indent=2))
