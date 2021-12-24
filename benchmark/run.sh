#!/bin/bash

CMD=""
for arg in "$@"; do
  shift
  [ $arg = '--' ] && break
  CMD="$CMD $arg"
done

run() {
  JSONFILE=$1
  BANNER=$2

  TIME=$(time $CMD "$JSONFILE" &>/dev/null 2>&1)
  if [ $? -ne 0 ]; then
    echo "Command Failed for $2."
    return
  fi

  filesize=$(du -sh "$JSONFILE" | awk '{print $1}')
  read lines characters <<< $(wc "$JSONFILE" | awk '{print $1, $3}')

  STATUS="filesize: $filesize | lines: $lines | characters: $characters"
  printf "$BANNER\n$STATUS\n$TIME\n"
}

for json_file in $(dirname "$0")/*.json;
do
  printf "$(run "$json_file" "file: $json_file")\n\n"
done

TMPFILE=$(mktemp)
trap "rm -rf ${TMPFILE}" EXIT HUP INT TERM

for url in "$@";
do
  curl "$url" > "$TMPFILE" 2>/dev/null
  printf "$(run "$TMPFILE" "Benchmarking url: $url")\n\n"
done
