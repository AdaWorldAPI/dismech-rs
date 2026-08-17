#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# upload-bake.sh — round-trip-verified S3 upload for a dismech-rs bake
# artifact. Mirrors AdaWorldAPI/MedCare-rs's scripts/upload-bake.sh
# byte-for-byte in method (same SigV4 approach, same PUT-then-GET-then-
# compare discipline, same credential-handling rules) with ONLY the key
# prefix changed: `dismech-rs/bakes/<tag>/<asset>` instead of
# `MedCare-rs/bakes/<tag>/<asset>`.
#
# WHY THE SAME METHOD, NOT A NEW ONE: a bake artifact that "uploaded
# successfully" per HTTP status but was truncated or hit a stale bucket
# policy is exactly the failure this discipline exists to catch BEFORE a
# consumer pins against it. There is no reason to re-derive that discipline
# per repo.
#
# Usage:
#   scripts/upload-bake.sh <tag> <local-file> [asset-name]
#
# Exit codes: 0 on a verified round-trip; 1 on any failure.
# Credentials: read from AWS_ENDPOINT_URL / AWS_S3_BUCKET_NAME /
# AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY, stripped of literal quotes
# INSIDE python, never printed/captured/logged.
# ---------------------------------------------------------------------------
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: $0 <tag> <local-file> [asset-name]" >&2
  exit 2
fi
TAG="$1"
SRC="$2"
NAME="${3:-$(basename "$SRC")}"

[ -f "$SRC" ] || { echo "upload-bake: no such file: $SRC" >&2; exit 1; }

HAVE_S3=0
if [ -n "${AWS_ENDPOINT_URL:-}" ] && [ -n "${AWS_S3_BUCKET_NAME:-}" ] && \
   [ -n "${AWS_ACCESS_KEY_ID:-}" ] && [ -n "${AWS_SECRET_ACCESS_KEY:-}" ]; then
  HAVE_S3=1
fi
if [ "$HAVE_S3" -ne 1 ]; then
  echo "upload-bake: S3 not configured (need AWS_ENDPOINT_URL, AWS_S3_BUCKET_NAME," >&2
  echo "  AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY) -- refusing to upload nowhere." >&2
  exit 1
fi

WANT="$(sha256sum "$SRC" | cut -d' ' -f1)"
echo "upload-bake: $SRC -> dismech-rs/bakes/$TAG/$NAME  ($WANT)" >&2

s3_put() {
  python3 - "$1" "$2" "$3" <<'PY'
import os, sys, hashlib, hmac, datetime, urllib.request, urllib.parse
tag, asset, path = sys.argv[1], sys.argv[2], sys.argv[3]
env = lambda k, d="": os.environ.get(k, d).strip().strip('"').strip("'")
ep, bucket = env("AWS_ENDPOINT_URL").rstrip("/"), env("AWS_S3_BUCKET_NAME")
region = env("AWS_DEFAULT_REGION", "auto") or "auto"
kid, secret = env("AWS_ACCESS_KEY_ID"), env("AWS_SECRET_ACCESS_KEY")
url = f"{ep}/{bucket}/dismech-rs/bakes/{tag}/{asset}"
u = urllib.parse.urlparse(url)
with open(path, "rb") as f:
    body = f.read()
payload_hash = hashlib.sha256(body).hexdigest()
now = datetime.datetime.now(datetime.timezone.utc)
amzdate, datestamp = now.strftime("%Y%m%dT%H%M%SZ"), now.strftime("%Y%m%d")
canon = (f"PUT\n{urllib.parse.quote(u.path, safe='/~')}\n\n"
         f"host:{u.netloc}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amzdate}\n\n"
         f"host;x-amz-content-sha256;x-amz-date\n{payload_hash}")
scope = f"{datestamp}/{region}/s3/aws4_request"
sts = f"AWS4-HMAC-SHA256\n{amzdate}\n{scope}\n{hashlib.sha256(canon.encode()).hexdigest()}"
k = ("AWS4" + secret).encode()
for part in (datestamp, region, "s3", "aws4_request"):
    k = hmac.new(k, part.encode(), hashlib.sha256).digest()
sig = hmac.new(k, sts.encode(), hashlib.sha256).hexdigest()
req = urllib.request.Request(url, data=body, method="PUT")
req.add_header("Authorization",
    f"AWS4-HMAC-SHA256 Credential={kid}/{scope}, "
    f"SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={sig}")
req.add_header("x-amz-date", amzdate)
req.add_header("x-amz-content-sha256", payload_hash)
req.add_header("Content-Length", str(len(body)))
try:
    op = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with op.open(req, timeout=300) as r:
        r.read()
except Exception as e:
    print(f"s3 put: {e}", file=sys.stderr)
    sys.exit(1)
PY
}

s3_get() {
  python3 - "$1" "$2" "$3" <<'PY'
import os, sys, hashlib, hmac, datetime, urllib.request, urllib.parse
tag, asset, out = sys.argv[1], sys.argv[2], sys.argv[3]
env = lambda k, d="": os.environ.get(k, d).strip().strip('"').strip("'")
ep, bucket = env("AWS_ENDPOINT_URL").rstrip("/"), env("AWS_S3_BUCKET_NAME")
region = env("AWS_DEFAULT_REGION", "auto") or "auto"
kid, secret = env("AWS_ACCESS_KEY_ID"), env("AWS_SECRET_ACCESS_KEY")
url = f"{ep}/{bucket}/dismech-rs/bakes/{tag}/{asset}"
u = urllib.parse.urlparse(url)
now = datetime.datetime.now(datetime.timezone.utc)
amzdate, datestamp = now.strftime("%Y%m%dT%H%M%SZ"), now.strftime("%Y%m%d")
payload = hashlib.sha256(b"").hexdigest()
canon = (f"GET\n{urllib.parse.quote(u.path, safe='/~')}\n\n"
         f"host:{u.netloc}\nx-amz-content-sha256:{payload}\nx-amz-date:{amzdate}\n\n"
         f"host;x-amz-content-sha256;x-amz-date\n{payload}")
scope = f"{datestamp}/{region}/s3/aws4_request"
sts = f"AWS4-HMAC-SHA256\n{amzdate}\n{scope}\n{hashlib.sha256(canon.encode()).hexdigest()}"
k = ("AWS4" + secret).encode()
for part in (datestamp, region, "s3", "aws4_request"):
    k = hmac.new(k, part.encode(), hashlib.sha256).digest()
sig = hmac.new(k, sts.encode(), hashlib.sha256).hexdigest()
req = urllib.request.Request(url)
req.add_header("Authorization",
    f"AWS4-HMAC-SHA256 Credential={kid}/{scope}, "
    f"SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={sig}")
req.add_header("x-amz-date", amzdate)
req.add_header("x-amz-content-sha256", payload)
try:
    op = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with op.open(req, timeout=300) as r, open(out, "wb") as f:
        while True:
            chunk = r.read(1 << 20)
            if not chunk:
                break
            f.write(chunk)
except Exception as e:
    print(f"s3 get: {e}", file=sys.stderr)
    sys.exit(1)
PY
}

if ! s3_put "$TAG" "$NAME" "$SRC"; then
  echo "upload-bake: PUT failed -- nothing to trust." >&2
  exit 1
fi

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
if ! s3_get "$TAG" "$NAME" "$TMP"; then
  echo "upload-bake: upload reported success but the READ-BACK failed." >&2
  exit 1
fi

GOT="$(sha256sum "$TMP" | cut -d' ' -f1)"
if [ "$GOT" != "$WANT" ]; then
  echo "upload-bake: ROUND-TRIP MISMATCH -- refusing to call this verified." >&2
  echo "  sent: $WANT" >&2
  echo "  got:  $GOT" >&2
  exit 1
fi

echo "upload-bake: round-trip verified (tag=$TAG, asset=$NAME)" >&2
echo "$WANT"
