#!/bin/sh
# cont-init: set up timezone and runtime directories before services start.
tz="${TZ:-UTC}"
if [ ! -f "/usr/share/zoneinfo/$tz" ]; then
    echo "[cont-init.d] Warning: Invalid timezone '$tz', using UTC"
    tz="UTC"
fi
ln -snf "/usr/share/zoneinfo/$tz" /etc/localtime
echo "$tz" > /etc/timezone
echo "[cont-init.d] Timezone set to: $tz"

mkdir -p /app/data /app/log
echo "[cont-init.d] Data and log directories ready"
