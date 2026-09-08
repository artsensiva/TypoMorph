#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

: "${TYPOMORPH_DEPLOY_HOST:?Set TYPOMORPH_DEPLOY_HOST to the FTPS hostname}"
: "${TYPOMORPH_DEPLOY_USER:?Set TYPOMORPH_DEPLOY_USER to the FTPS username}"
: "${TYPOMORPH_DEPLOY_PASS:?Set TYPOMORPH_DEPLOY_PASS to the FTPS password}"
HOST="${TYPOMORPH_DEPLOY_HOST}"
FTP_USER="${TYPOMORPH_DEPLOY_USER}"
FTP_PASS="${TYPOMORPH_DEPLOY_PASS}"
SITE_URL="${TYPOMORPH_DEPLOY_URL:-https://${HOST}/typomorph}"

echo "=== 1. Сборка продакшен-архива ==="
bash "${ROOT_DIR}/scripts/build-production-tarball.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

tar -xzf "${ROOT_DIR}/typomorph-production.tar.gz" -C "${TMP_DIR}"

echo "=== 2. Выгрузка на сервер по FTPS (TLS) ==="
lftp -c "
set ftp:ssl-force true
set ssl:verify-certificate no
open -u ${FTP_USER},${FTP_PASS} ${HOST}
cd public_html
mkdir -p typomorph
mirror -R ${TMP_DIR} typomorph
put ${TMP_DIR}/install.sh -o install.sh
bye
"

echo "=== 3. Проверка доступности ==="
echo -n "Лендинг: "
curl -sSL -o /dev/null -w "%{http_code}\n" "${SITE_URL}/index.html"

echo -n "Installer: "
curl -sSL -o /dev/null -w "%{http_code}\n" "${SITE_URL}/install.sh"

echo "Деплой успешно завершен!"
