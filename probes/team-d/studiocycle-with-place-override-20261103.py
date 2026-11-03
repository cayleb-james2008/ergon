#!/usr/bin/env python3
"""studiocycle — close/reopen the fleet's Roblox Studio without the owner.

Why: the Studio process holds a per-process Play budget (Team D's runner allows 3 Plays per
Studio process for crash safety), so every opening must end with a Studio close + relaunch.
The owner asked not to be the one doing it. Closing a *dirty* place normally pops a
"Save changes?" dialog, and answering it blind is dangerous: a stray "Save" would overwrite
the sealed place file.

Method: request a graceful WM close; if the window survives (i.e. the dialog appeared), kill
the Studio process instead — the dialog is never answered, so nothing is ever saved. The sealed
place file is hashed before and after; if it changed for any reason, the preserved backup is
restored and the run fails loudly.

Safety:
  * only the allowlisted Studio window is touched (via winctl);
  * only the flatpak app id is killed, never other Wine/Steam processes;
  * the place file is hash-guarded with a backup and a post-close restore check;
  * every action is logged to ~/.local/state/winctl/actions.jsonl.

Usage:
  studiocycle snapshot                 # record the sealed place hash
  studiocycle close --discard          # close Studio; never save; verify place unchanged
  studiocycle open                     # relaunch the place and fit the window
  studiocycle cycle                    # close --discard, then open
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

VINEGAR_DATA = Path("/home/cayleb/.var/app/org.vinegarhq.Vinegar/data")
DEFAULT_PLACE = VINEGAR_DATA / "usg" / "usg-desktop.rbxlx"


def resolve_place(environ=None) -> Path:
    """The place this tool guards. Default: the sealed slot. STUDIOCYCLE_PLACE selects another slot (2026-11-03).

    The override is validated so it can only name a *.rbxlx under the Vinegar data dir; each slot keeps its own
    sealed-bytes backup (see backup_for), so using one slot never rewrites another slot's backup.
    """
    environ = os.environ if environ is None else environ
    raw = environ.get("STUDIOCYCLE_PLACE")
    if not raw:
        return DEFAULT_PLACE
    path = Path(raw)
    if not path.is_absolute() or path.suffix != ".rbxlx" or ".." in path.parts or VINEGAR_DATA not in path.parents:
        raise SystemExit(f"studiocycle: STUDIOCYCLE_PLACE must be an absolute *.rbxlx under {VINEGAR_DATA}: {raw}")
    real = path.resolve()  # symlinks are followed: a slot may never lead outside the data dir or onto the sealed slot
    same = real.exists() and DEFAULT_PLACE.exists() and os.path.samefile(real, DEFAULT_PLACE)  # hardlinks too
    if VINEGAR_DATA.resolve() not in real.parents or real == DEFAULT_PLACE.resolve() or same:
        raise SystemExit(f"studiocycle: STUDIOCYCLE_PLACE resolves outside the data dir or onto the sealed slot: {raw} -> {real}")
    return path


def backup_for(place: Path, sealed_dir: Path) -> Path:
    if place == DEFAULT_PLACE:
        return sealed_dir / "usg-desktop.rbxlx.sealed.bak"
    tag = hashlib.sha256(str(place.parent).encode()).hexdigest()[:8]  # distinct directories never share a backup
    return sealed_dir / f"{place.parent.name}-{tag}--{place.name}.sealed.bak"


PLACE = resolve_place()
PLACE_WIN = "Z:" + str(PLACE).replace("/", "\\")
FLATPAK_ID = "org.vinegarhq.Vinegar"
STUDIO_TARGET = "studio"
FIT_W, FIT_H = 2200, 1000
STATE_DIR = Path.home() / ".local" / "state" / "winctl"
SEALED_DIR = STATE_DIR / "sealed"
LOG_PATH = STATE_DIR / "actions.jsonl"
BACKUP = backup_for(PLACE, SEALED_DIR)
LAUNCH_LOG = Path("/tmp/studio-launch.log")
WINCTL = str(Path.home() / ".local" / "bin" / "winctl")
AUTOSAVE_DIR = Path(
    "/home/cayleb/.var/app/org.vinegarhq.Vinegar/data/vinegar/prefixes/studio/drive_c/"
    "users/cayleb/AppData/Local/Roblox/RobloxStudio/AutoSaves"
)
DISCARD_DIR = STATE_DIR / "discarded-autosaves"


class CycleError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def log(record: dict) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    try:
        os.chmod(STATE_DIR, 0o700)
    except OSError:
        pass
    record = {"ts": datetime.now(timezone.utc).isoformat(timespec="seconds"), **record}
    fd = os.open(LOG_PATH, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        os.write(fd, (json.dumps(record, sort_keys=True) + "\n").encode())
    finally:
        os.close(fd)


def hyprctl_json(*args: str) -> list[dict]:
    proc = subprocess.run(
        ["hyprctl", "-j", *args], capture_output=True, text=True, timeout=10
    )
    if proc.returncode != 0:
        raise CycleError(proc.stderr.strip() or "hyprctl failed")
    return json.loads(proc.stdout or "[]")


def studio_windows() -> list[dict]:
    return [
        c
        for c in hyprctl_json("clients")
        if (c.get("class") or "") == "steam_proton" and c.get("mapped")
    ]


def winctl_json(*args: str) -> dict:
    proc = subprocess.run(
        [WINCTL, *args], capture_output=True, text=True, timeout=20
    )
    if proc.returncode != 0:
        raise CycleError(proc.stderr.strip() or f"winctl {args[0]} failed")
    return json.loads(proc.stdout)


def dispatch_close(address: str) -> None:
    expr = 'hl.dsp.window.close({ window = "address:%s" })' % address
    proc = subprocess.run(
        ["hyprctl", "dispatch", expr], capture_output=True, text=True, timeout=10
    )
    if not proc.stdout.strip().startswith("ok"):
        raise CycleError(proc.stdout.strip() or "close dispatch failed")


def wait_gone(timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not studio_windows():
            return True
        time.sleep(0.2)
    return False


def wait_present(timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if studio_windows():
            return True
        time.sleep(0.3)
    return False


def cmd_snapshot() -> int:
    if not PLACE.is_file():
        print(f"studiocycle: place file missing: {PLACE}", file=sys.stderr)
        return 2
    digest = sha256(PLACE)
    SEALED_DIR.mkdir(parents=True, exist_ok=True)
    try:
        os.chmod(SEALED_DIR, 0o700)
    except OSError:
        pass
    if not BACKUP.is_file() or sha256(BACKUP) != digest:
        shutil.copy2(PLACE, BACKUP)
        os.chmod(BACKUP, 0o600)
    rec = {"action": "snapshot", "place": str(PLACE), "sha256": digest}
    log(rec)
    print(json.dumps(rec))
    return 0


def ensure_backup(current: str) -> None:
    SEALED_DIR.mkdir(parents=True, exist_ok=True)
    if not BACKUP.is_file():
        shutil.copy2(PLACE, BACKUP)
        os.chmod(BACKUP, 0o600)
        log({"action": "backup-created", "sha256": sha256(BACKUP)})


def discard_autosaves() -> list[str]:
    """Move the place's crash-recovery autosaves aside.

    A killed Studio leaves `usg-desktop_AutoRecovery_*.rbxl` behind, and the next launch then
    shows a recovery dialog that would need blind input. Moving them aside (preserved, never
    deleted) means the next launch opens the sealed place directly.
    """
    moved: list[str] = []
    if not AUTOSAVE_DIR.is_dir():
        return moved
    DISCARD_DIR.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    for path in sorted(AUTOSAVE_DIR.glob("usg-desktop_AutoRecovery_*.rbxl")):
        dest = DISCARD_DIR / f"{stamp}-{path.name}"
        try:
            shutil.move(str(path), str(dest))
            moved.append(dest.name)
        except OSError:
            pass
    if moved:
        log({"action": "discard-autosaves", "moved": moved})
    return moved


def cmd_close(discard: bool) -> int:
    if not discard:
        print("studiocycle: refusing without --discard (a save is never allowed)", file=sys.stderr)
        return 4
    if not PLACE.is_file():
        print(f"studiocycle: place file missing: {PLACE}", file=sys.stderr)
        return 2
    before = sha256(PLACE)
    ensure_backup(before)
    wins = studio_windows()
    if not wins:
        print(json.dumps({"action": "close", "result": "no-studio-window"}))
        return 0
    address = wins[0].get("address")
    dialog_seen = False
    try:
        dispatch_close(address)
        if wait_gone(4.0):
            method = "graceful"
        else:
            dialog_seen = True
            # A dialog is up. Kill the app instead of answering it: nothing is ever saved.
            subprocess.run(
                ["flatpak", "kill", FLATPAK_ID],
                capture_output=True,
                text=True,
                timeout=20,
            )
            if not wait_gone(12.0):
                # last resort: SIGKILL the Studio process by pid
                pid = wins[0].get("pid")
                if pid:
                    subprocess.run(["kill", "-9", str(pid)], check=False)
                if not wait_gone(6.0):
                    raise CycleError("Studio did not exit after close+kill")
            method = "killed-after-dialog"
    except CycleError as exc:
        log({"action": "close", "result": "error", "error": str(exc)})
        print(f"studiocycle: {exc}", file=sys.stderr)
        return 5
    discarded = discard_autosaves()
    after = sha256(PLACE)
    restored = False
    if after != before:
        # Something saved. Restore the sealed bytes and fail loudly.
        if not BACKUP.is_file():
            log({"action": "close", "result": "place-changed-no-backup"})
            print("studiocycle: PLACE CHANGED and no backup exists", file=sys.stderr)
            return 3
        shutil.copy2(BACKUP, PLACE)
        restored_sha = sha256(PLACE)
        restored = restored_sha == sha256(BACKUP)
        log(
            {
                "action": "close",
                "result": "place-changed-restored",
                "before": before,
                "after": after,
                "restored": restored,
            }
        )
        print(
            json.dumps(
                {
                    "action": "close",
                    "result": "place-changed-restored",
                    "before": before,
                    "after": after,
                    "restored": restored,
                }
            )
        )
        return 3
    rec = {
        "action": "close",
        "result": "ok",
        "method": method,
        "dialog_seen": dialog_seen,
        "discarded_autosaves": discarded,
        "place_sha256": after,
        "sealed_unchanged": True,
    }
    log(rec)
    print(json.dumps(rec))
    return 0


def cmd_open() -> int:
    if studio_windows():
        print(json.dumps({"action": "open", "result": "already-open"}))
        return 0
    before = sha256(PLACE) if PLACE.is_file() else None
    env = dict(os.environ)
    logfh = open(LAUNCH_LOG, "ab")
    subprocess.Popen(
        ["setsid", "nohup", "flatpak", "run", FLATPAK_ID, PLACE_WIN],
        stdout=logfh,
        stderr=logfh,
        stdin=subprocess.DEVNULL,
        env=env,
        start_new_session=True,
    )
    if not wait_present(90.0):
        print(json.dumps({"action": "open", "result": "timeout-waiting-for-window"}))
        return 5
    fit: dict = {}
    deadline = time.monotonic() + 25.0
    while time.monotonic() < deadline:
        try:
            candidate = winctl_json(
                "set-size", "--target", STUDIO_TARGET, "--width", str(FIT_W), "--height", str(FIT_H), "--json"
            )
            if candidate.get("after"):
                fit = candidate
                break
            fit = candidate
        except CycleError as exc:
            fit = {"error": str(exc)}
        time.sleep(1.0)
    after = sha256(PLACE) if PLACE.is_file() else None
    rec = {
        "action": "open",
        "result": "ok",
        "place_sha256_before": before,
        "place_sha256_after": after,
        "fit": fit.get("result", fit),
        "window": fit.get("after"),
    }
    log(rec)
    print(json.dumps(rec))
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="studiocycle")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("snapshot")
    p_close = sub.add_parser("close")
    p_close.add_argument("--discard", action="store_true")
    sub.add_parser("open")
    sub.add_parser("cycle")
    args = ap.parse_args(argv)
    if args.cmd == "snapshot":
        return cmd_snapshot()
    if args.cmd == "close":
        return cmd_close(args.discard)
    if args.cmd == "open":
        return cmd_open()
    if args.cmd == "cycle":
        rc = cmd_close(True)
        if rc not in (0,):
            return rc
        return cmd_open()
    return 5


if __name__ == "__main__":
    sys.exit(main())
