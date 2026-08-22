#!/bin/sh
# Orcker GUI .rpm %post scriptlet.
#
# Tauri's rpm bundler installs the GUI **and** its externalBin sidecars (orcker,
# orckerd, orcker-helper) side-by-side into /usr/bin - so they are already on PATH and
# already siblings of /usr/bin/orcker-gui. No symlinking is needed in that (normal)
# layout; the only post-install work is granting the daemon permission to bind
# privileged ports (80/443). If setcap fails (overlayfs/NFS/noxattr mounts can't
# hold file capabilities) the daemon falls back to 8080/8443.
#
# Unlike a deb postinst (which receives the `configure` verb as $1), an rpm %post
# receives $1 = the post-transaction count of packages with this name: 1 on a
# fresh install, 2 during an upgrade. We want to (re)apply setcap in both cases
# (rpm wipes file caps on upgrade), so there is no verb to switch on. A
# /usr/lib/<product>/ fallback is kept for resilience with foreign layouts that
# stage the sidecars there instead of /usr/bin.
set -e

# Locate the daemon: /usr/bin (normal) or a single /usr/lib/<dir>/ fallback we
# symlink onto PATH. Fail closed (below) only if it's absent from both.
orckerd=""
if [ -x /usr/bin/orckerd ] && [ -x /usr/bin/orcker ] && [ -x /usr/bin/orcker-helper ]; then
  orckerd=/usr/bin/orckerd
else
  # Locate the single embedded dir holding all three binaries; refuse on an
  # ambiguous match (a stale/foreign tree) before touching /usr/bin.
  dir=""
  for cand in /usr/lib/*/orckerd; do
    [ -f "$cand" ] || continue
    d=$(dirname "$cand")
    [ -f "$d/orcker" ] && [ -f "$d/orcker-helper" ] || continue
    if [ -n "$dir" ] && [ "$dir" != "$d" ]; then
      echo "orcker: multiple embedded binary dirs ($dir and $d); refusing to symlink" >&2
      exit 1
    fi
    dir="$d"
  done
  if [ -n "$dir" ]; then
    # Co-locate on PATH; refuse to clobber a real file or a foreign symlink at
    # /usr/bin/$b - that would steal a path owned by another package.
    for b in orcker orckerd orcker-helper; do
      src="$dir/$b"
      dst="/usr/bin/$b"
      if [ -e "$dst" ] && [ ! -L "$dst" ]; then
        echo "orcker: $dst exists and is not a symlink; refusing to overwrite" >&2
        exit 1
      fi
      if [ -L "$dst" ] && [ "$(readlink "$dst")" != "$src" ]; then
        echo "orcker: $dst points elsewhere; refusing to overwrite foreign symlink" >&2
        exit 1
      fi
      ln -sfn "$src" "$dst"
    done
    orckerd="$dir/orckerd"
  fi
fi
if [ -z "$orckerd" ]; then
  echo "orcker: could not locate the orckerd binary in /usr/bin or /usr/lib" >&2
  exit 1
fi

# Privileged-port capability on the REAL daemon binary; best-effort.
if command -v setcap >/dev/null 2>&1; then
  setcap 'cap_net_bind_service=+ep' "$orckerd" \
    || echo "orcker: setcap failed; the daemon will use ports 8080/8443" >&2
else
  echo "orcker: setcap not found (install libcap); the daemon will use 8080/8443" >&2
fi

exit 0
