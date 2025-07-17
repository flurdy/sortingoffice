#!/usr/bin/env bash

CONFIG_FILE="config/config.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Config file not found: $CONFIG_FILE" >&2
  exit 1
fi

echo -e "ID\tLabel\tURL"
awk '
  /^\[\[databases\]\]/ {in_db=1; id=""; label=""; url=""}
  in_db && /^id *=/     {gsub(/"/, "", $3); id=$3}
  in_db && /^label *=/  {gsub(/"/, "", $3); label=$3}
  in_db && /^url *=/    {gsub(/"/, "", $3); url=$3}
  in_db && /^url *=/ {
    print id "\t" label "\t" url
    in_db=0
  }
' "$CONFIG_FILE" 
