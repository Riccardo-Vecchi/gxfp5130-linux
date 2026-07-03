#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run as root: sudo $0" >&2
  exit 1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
upstreams="$root/.upstreams"
kernel_src="$upstreams/gxfp_linux_driver"
moc_src="$upstreams/gxfpmoc"
open_fprintd_src="$upstreams/open-fprintd"

if [[ ! -d "$kernel_src" || ! -d "$moc_src" || ! -d "$open_fprintd_src" ]]; then
  echo "Missing upstreams. Run scripts/fetch-upstreams.sh as your normal user first." >&2
  exit 1
fi

command -v dkms >/dev/null
command -v cargo >/dev/null
command -v cmake >/dev/null

install -d -m 0755 /usr/src/gxfp-0.1.0
rsync -a --delete --exclude .git "$kernel_src/" /usr/src/gxfp-0.1.0/
install -m 0644 "$root/scripts/gxfp.dkms.conf" /usr/src/gxfp-0.1.0/dkms.conf

dkms remove -m gxfp -v 0.1.0 --all >/dev/null 2>&1 || true
dkms add -m gxfp -v 0.1.0
dkms build -m gxfp -v 0.1.0
dkms install -m gxfp -v 0.1.0

printf '%s\n' gxfp > /etc/modules-load.d/gxfp.conf
cat > /etc/modprobe.d/blacklist-sil6250-gxfp.conf <<'EOF'
# GXFP5130 is handled by the gxfp driver. Keep experimental sil6250 variants
# from racing this driver if they were locally patched to claim GXFP5130.
blacklist sil6250
EOF

cmake -S "$moc_src" -B "$moc_src/build" -DCMAKE_BUILD_TYPE=Release
cmake --build "$moc_src/build" --target gxfp_capture_once gxfp_psk_tool -j"$(nproc)"
install -m 0755 "$moc_src/build/gxfp_capture_once" /usr/local/bin/gxfp_capture_once
install -m 0755 "$moc_src/build/gxfp_psk_tool" /usr/local/bin/gxfp_psk_tool

( cd "$open_fprintd_src" && python3 setup.py install --force --prefix=/usr --root=/ )
install -m 0644 "$open_fprintd_src/dbus_service/net.reactivated.Fprint.conf" /etc/dbus-1/system.d/net.reactivated.Fprint.conf
install -m 0644 "$open_fprintd_src/dbus_service/net.reactivated.Fprint.service" /usr/share/dbus-1/system-services/net.reactivated.Fprint.service
install -m 0644 "$open_fprintd_src/debian/open-fprintd.service" /etc/systemd/system/open-fprintd.service
install -m 0644 "$root/dbus/io.github.uunicorn.Fprint.conf" /etc/dbus-1/system.d/io.github.uunicorn.Fprint.conf

cargo build --release --manifest-path "$root/openfprintd-backend/Cargo.toml" --bins
install -m 0755 "$root/openfprintd-backend/target/release/gxfp-openfprintd" /usr/local/bin/gxfp-openfprintd
install -m 0755 "$root/openfprintd-backend/target/release/enroll_from_pgms" /usr/local/bin/gxfp-enroll-from-pgms
install -m 0755 "$root/openfprintd-backend/target/release/analyze_samples" /usr/local/bin/gxfp-analyze-samples

install -d -m 0700 /var/lib/open-fprintd/gxfp
install -d -m 0755 /usr/local/libexec
install -m 0755 "$root/scripts/gxfp-ensure-device" /usr/local/libexec/gxfp-ensure-device
install -m 0644 "$root/systemd/gxfp-openfprintd.service" /etc/systemd/system/gxfp-openfprintd.service

systemctl daemon-reload
systemctl mask fprintd.service >/dev/null 2>&1 || true
systemctl enable --now open-fprintd.service
systemctl enable --now gxfp-openfprintd.service

echo "Installed GXFP backend. Ensure /var/lib/open-fprintd/gxfp/psk-new-raw32.bin exists before enrollment."
