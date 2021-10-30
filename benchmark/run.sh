#!/bin/bash

CMD="$1"
shift

run() {
  json_file=$1
  filesize=$(du -sh "$json_file" | awk '{print $1}')
  read lines characters <<< $(wc "$json_file" | awk '{print $1, $3}')

  TIME=$(time $CMD "$json_file" &>/dev/null 2>&1)
  if [ $? -ne 0 ]; then
    echo "Command Failed for '$2'."
    return
  fi

  BANNER="Benchmarking $2"
  echo "$BANNER"
  STATUS="filesize: $filesize | lines: $lines | characters: $characters"
  echo "$STATUS"
  echo "$TIME"
}

for json_file in $(dirname "$0")/*.json;
do
  printf "$(run "$json_file" "file: $json_file")\n\n"
done

TMPFILE=$(mktemp)
trap "rm -rf ${TMPFILE}" EXIT HUP INT TERM

for url in "$@";
do
  curl "$url" > "$TMPFILE" 2> /dev/null
  printf "$(run "$TMPFILE" "url: $url")\n\n"
done
