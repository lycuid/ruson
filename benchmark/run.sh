#!/bin/bash

function run {
  json_file=$1
  filesize=$(du -sh "$json_file" | awk '{print $1}')
  read lines characters <<< $(wc "$json_file" | awk '{print $1, $3}')

  BANNER=$2
  echo "$BANNER"
  STATUS="filesize: $filesize | lines: $lines | characters: $characters"
  echo "$STATUS"
  COMMAND=$((time ./target/release/ruson "$json_file" &>/dev/null) 2>&1)
  echo "$COMMAND"
}

for json_file in $(ls ./benchmark/*.json);
do
  printf "$(run "$json_file" "Benchmarking $json_file")\n\n"
done

TMPFILE=$(mktemp)
trap "rm -rf ${TMPFILE}" EXIT HUP INT TERM

for url in "$@";
do
  curl "$url" > "$TMPFILE" 2> /dev/null
  printf "$(run "$TMPFILE" "json url: $url")\n\n"
done
