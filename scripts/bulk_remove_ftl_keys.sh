#!/usr/bin/env bash
# Remove a list of translation keys from all .ftl files in resources/locales/

KEYS_FILE="orphaned_keys.txt"
LOCALES_DIR="resources/locales"

if [ ! -f "$KEYS_FILE" ]; then
  echo "Key list file $KEYS_FILE not found!"
  exit 1
fi

for ftl in $LOCALES_DIR/*/messages.ftl; do
  cp "$ftl" "$ftl.bak"
  tmpfile=$(mktemp)
  cp "$ftl" "$tmpfile"
  while read -r key; do
    # Remove lines starting with the key followed by optional whitespace and =
    sed -i "/^$key\s*=.*/d" "$tmpfile"
  done < "$KEYS_FILE"
  mv "$tmpfile" "$ftl"
  echo "Cleaned $ftl"
done

echo "Done. Backups saved as .bak files." 
