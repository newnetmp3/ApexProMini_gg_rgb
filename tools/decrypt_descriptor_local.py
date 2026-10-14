#!/usr/bin/env python3
"""Decrypt personally supplied GG .edevice locally with steelpatcher, never publishing secrets.

Prerequisites: installed 'cryptography' and source import for
https://github.com/hurryman2212/steelpatcher (e.g. PYTHONPATH=../steelpatcher).
"""
import argparse
import hashlib
import os
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
PRIVATE = PROJECT / ".private"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", required=True, type=Path, help="Local Windows SteelSeriesEngine.exe")
    parser.add_argument("--descriptor", required=True, type=Path, help="Local encrypted .edevice")
    args = parser.parse_args()

    try:
        from ggpatcher import descriptor
    except ImportError as exc:
        parser.error("Install cryptography and set PYTHONPATH to your steelpatcher checkout: " + str(exc))

    engine_bytes = args.engine.read_bytes()
    encrypted = args.descriptor.read_bytes()
    # Recovered key remains only in this process's memory; it is NEVER printed or saved.
    passphrase = descriptor.extract_descriptor_password(engine_bytes)
    plaintext = descriptor.decrypt_descriptor(encrypted, passphrase)

    PRIVATE.mkdir(mode=0o700, exist_ok=True)
    PRIVATE.chmod(0o700)
    out = PRIVATE / (args.descriptor.name + ".scm")
    # Exclusive creation avoids accidental overwrite; mode 0600 limits access to owner.
    fd = os.open(out, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "wb") as stream:
        stream.write(plaintext)
    print(f"Decrypted locally to {out} ({len(plaintext)} bytes)")
    print("SHA-256:", hashlib.sha256(plaintext).hexdigest())
    print("Do not commit, share, or publish this proprietary device definition.")


if __name__ == "__main__":
    main()
