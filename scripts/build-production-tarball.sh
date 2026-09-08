#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARBALL="${ROOT_DIR}/typomorph-production.tar.gz"
TMP_STAGE="$(mktemp -d)"
trap 'rm -rf "${TMP_STAGE}"' EXIT

echo "--> Сборка Debian пакета..."
bash "${ROOT_DIR}/scripts/build-deb.sh"

echo "--> Подготовка структуры лендинга и дистрибутива..."
mkdir -p "${TMP_STAGE}/downloads"

# Копируем статику лендинга и скрипт установки
cp -r "${ROOT_DIR}/landing/"* "${TMP_STAGE}/"
cp "${ROOT_DIR}/landing/install.sh" "${TMP_STAGE}/install.sh"

# Копируем собранные deb-пакеты в downloads
find "${ROOT_DIR}" -maxdepth 2 -name "*.deb" -exec cp {} "${TMP_STAGE}/downloads/" \;

# Создаем локальный .htaccess для typomorph
cat << 'HTACCESS' > "${TMP_STAGE}/.htaccess"
<IfModule mod_rewrite.c>
RewriteEngine Off
</IfModule>
HTACCESS

echo "--> Упаковка в ${TARBALL}..."
tar -czf "${TARBALL}" -C "${TMP_STAGE}" .
echo "Архив успешно собран: $(ls -lh "${TARBALL}" | awk '{print $5, $9}')"
