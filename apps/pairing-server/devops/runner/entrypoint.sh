#!/bin/sh
# Prepare credentials and run the deployment playbook.
#
# Windows bind-mounts carry no POSIX permissions, so SSH keys and the vault
# password file are copied into the container and chmod 600'd before use.
# /ssh and /vault are mounted read-only by deploy.ps1 / deploy.sh.
set -e

ENV_NAME="$1"
shift

mkdir -p /root/.ssh
cp /ssh/* /root/.ssh/ 2>/dev/null || true
chmod 700 /root/.ssh
chmod 600 /root/.ssh/* 2>/dev/null || true

PW_FILE="/tmp/dropvoice-${ENV_NAME}.pwd"
cp "/vault/dropvoice-${ENV_NAME}.pwd" "$PW_FILE"
chmod 600 "$PW_FILE"

exec ansible-playbook -i hosts.yml deploy.yml -l "$ENV_NAME" --vault-password-file "$PW_FILE" "$@"
