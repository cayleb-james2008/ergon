#!/usr/bin/python3
"""CHECKER harness: drive `<installed>/bin/dsh-fleet.mjs first-run --credentials` under a REAL PTY.

Sends the fake value at the first provider prompt and a bare Enter (skip) at the rest.
The fake value is read from --value-file and is never printed by this script; the raw
PTY transcript is written to --transcript for the verifier to scan.
Exit 0 only when the CLI exit code is 0 and the value did NOT appear in the transcript.
"""
import argparse
import os
import pty
import select
import subprocess
import sys
import time

PROMPT = "[Enter to skip]:"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--node", required=True)
    ap.add_argument("--cli", required=True)
    ap.add_argument("--store", required=True)
    ap.add_argument("--socket", required=True)
    ap.add_argument("--dsh-home", required=True)
    ap.add_argument("--value-file", required=True)
    ap.add_argument("--transcript", required=True)
    ap.add_argument("--timeout", type=int, default=120)
    a = ap.parse_args()

    with open(a.value_file, encoding="utf-8") as fh:
        fake = fh.read().strip()
    if not fake or "\n" in fake or any(ord(c) < 32 for c in fake):
        print("BAD value file")
        return 2

    env = dict(os.environ)
    env.update({
        "DSH_FLEET_CREDENTIALS_FILE": a.store,
        "DSH_FLEET_SOCKET": a.socket,
        "DSH_HOME": a.dsh_home,
        "TERM": "xterm-256color",
    })

    master, slave = pty.openpty()
    proc = subprocess.Popen(
        [a.node, a.cli, "first-run", "--credentials"],
        stdin=slave, stdout=slave, stderr=slave, env=env,
        close_fds=True, start_new_session=True,
    )
    os.close(slave)
    buf = bytearray()
    answered = 0
    responses = []
    deadline = time.time() + a.timeout
    try:
        while True:
            remaining = deadline - time.time()
            if remaining <= 0:
                proc.kill()
                print("TIMEOUT")
                break
            r, _, _ = select.select([master], [], [], min(1.0, max(0.0, remaining)))
            if r:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    chunk = b""
                if not chunk:
                    break
                buf += chunk
            seen = buf.decode("utf-8", "replace").count(PROMPT)
            if seen > answered:
                if answered == 0:
                    os.write(master, (fake + "\r").encode())
                    responses.append("value")
                else:
                    os.write(master, b"\r")
                    responses.append("enter")
                answered += 1
            if proc.poll() is not None:
                end = time.time() + 3
                while time.time() < end:
                    r, _, _ = select.select([master], [], [], 0.2)
                    if not r:
                        break
                    try:
                        chunk = os.read(master, 65536)
                    except OSError:
                        break
                    if not chunk:
                        break
                    buf += chunk
                break
    finally:
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()
            proc.wait()
        try:
            os.close(master)
        except OSError:
            pass

    transcript = bytes(buf)
    with open(a.transcript, "wb") as fh:
        fh.write(transcript)
    text = transcript.decode("utf-8", "replace")
    print(f"exit_code={proc.returncode}")
    print(f"prompts_seen={answered}")
    print(f"responses={responses}")
    print(f"transcript_bytes={len(transcript)}")
    print(f"value_in_transcript={'YES' if fake in text else 'NO'}")
    print(f"old_error_in_transcript={'YES' if 'terminal secret reader' in text else 'NO'}")
    return 0 if (proc.returncode == 0 and fake not in text and "terminal secret reader" not in text) else 1


if __name__ == "__main__":
    raise SystemExit(main())
