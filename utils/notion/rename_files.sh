#!/bin/bash

#this script is to remove the md5 hash from folder/file name generated by notion


if [ -z "$1" ]; then
  echo "Usage: $0 <directory_path>"
  exit 1
fi

DIRECTORY="$1"

if [ ! -d "$DIRECTORY" ]; then
  echo "Directory not found. Please enter a valid directory path."
  exit 1
fi

rename_items() {
  for item in "$1"/*; do
    base_name=$(basename "$item")
    dir_name=$(dirname "$item")

    new_name=$(echo "$base_name" | sed -E 's/ [a-f0-9]{32}//')

    if [ "$base_name" != "$new_name" ]; then
      mv "$item" "$dir_name/$new_name"
      echo "Renamed: $item -> $dir_name/$new_name"
      item="$dir_name/$new_name"
    fi

    if [ -d "$item" ]; then
      rename_items "$item"
    fi
  done
}

rename_items "$DIRECTORY"
