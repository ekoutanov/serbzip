#!/bin/bash
set -e

type bc >/dev/null 2>&1 || { echo >&2 "bc is not installed; aborting."; exit 1; }
type gzip >/dev/null 2>&1 || { echo >&2 "gzip is not installed; aborting."; exit 1; }
type bzip2 >/dev/null 2>&1 || { echo >&2 "bzip2 is not installed; aborting."; exit 1; }
type cargo >/dev/null 2>&1 || { echo >&2 "cargo is not installed; aborting."; exit 1; }

base_dir="$(dirname "$0")"
test_data_dir=${base_dir}/../test_data

temp_dir=${TMPDIR-/tmp}

RED='\033[0;31m'
GREEN='\033[0;32m'
RESET='\033[0m'

echo "|filename                      |size      |words     |gzip size |bzip2 size|sz size   |sz reduction %|sz.gz size  |sz+gz reduction %|sz.bz2 size |sz+bz2 reduction %|"
echo "|-----------------------------:|---------:|---------:|---------:|---------:|---------:|-------------:|-----------:|----------------:|-----------:|-----------------:|"

main_test_files=$(ls -Sr $test_data_dir | grep -v "dict")
additional_test_files="../README.md"

files="$main_test_files $additional_test_files"
for file in $files; do
  # basic file stats
  test_file=${test_data_dir}/$file
  raw_bytes=$(cat ${test_file} | wc -c)
  words=$(cat ${test_file} | wc -w)
  echo -n "|$(printf %30s $file)|$(printf %10d $raw_bytes)|$(printf %10d $words)"

  file=$(basename $file)

  # compress with gzip
  cp $test_file $temp_dir
  rm ${temp_dir}/${file}."gz" 2> /dev/null || true
  gzip --best --keep -q ${temp_dir}/${file}
  gzip_bytes=$(cat ${temp_dir}/${file}."gz" | wc -c)
  echo -n "|$(printf %10d ${gzip_bytes})"
  rm ${temp_dir}/${file}."gz"

  # compress with bzip2
  cp $test_file $temp_dir
  rm ${temp_dir}/${file}."bz2" 2> /dev/null || true
  bzip2 --best --keep -q ${temp_dir}/${file}
  bzip2_bytes=$(cat ${temp_dir}/${file}.bz2 | wc -c)
  echo -n "|$(printf %10d ${bzip2_bytes})"
  rm ${temp_dir}/${file}."bz2"

  # compress with serbzip
  cargo -q run -- --quiet --compress --input-file $test_file --output-file ${temp_dir}/${file}."sz"
  sz_bytes=$(cat ${temp_dir}/${file}."sz" | wc -c)
  echo -n "|$(printf %10d ${sz_bytes})"

  # calculate [raw]->[sz] size reduction
  sz_reduction=$(echo "scale=2; 100 * ($raw_bytes - $sz_bytes)/$raw_bytes" | bc)
  if (( $(echo "$sz_reduction < 0" | bc -l) )); then echo -en "|$RED"; else echo -en "|$GREEN"; fi
  echo -en "$(printf %14s ${sz_reduction})$RESET"

  # compress sz output with gzip
  rm ${temp_dir}/${file}."sz.gz" 2> /dev/null || true
  gzip --best --keep -q ${temp_dir}/${file}."sz"
  sz_gzip_bytes=$(cat ${temp_dir}/${file}."sz.gz" | wc -c)
  echo -n "|$(printf %12d ${sz_gzip_bytes})"
  rm ${temp_dir}/${file}."sz.gz"

  # calculate [raw.gz]->[sz.gz] size reduction
  sz_gzip_reduction=$(echo "scale=2; 100*($gzip_bytes - $sz_gzip_bytes)/$gzip_bytes" | bc)
  if (( $(echo "$sz_gzip_reduction < 0" | bc -l) )); then echo -en "|$RED"; else echo -en "|$GREEN"; fi
  echo -en "$(printf %17s ${sz_gzip_reduction})$RESET"

  # compress sz output with bzip2
  rm ${temp_dir}/${file}."sz.bz2" 2> /dev/null || true
  bzip2 --best --keep -q ${temp_dir}/${file}."sz"
  sz_bzip2_bytes=$(cat ${temp_dir}/${file}."sz.bz2" | wc -c)
  echo -n "|$(printf %12d ${sz_bzip2_bytes})"
  rm ${temp_dir}/${file}."sz.bz2"

  # calculate [raw.bz2]->[sz.bz2] size reduction
  sz_bzip2_reduction=$(echo "scale=2; 100*($bzip2_bytes - $sz_bzip2_bytes)/$bzip2_bytes" | bc)
  if (( $(echo "$sz_bzip2_reduction < 0" | bc -l) )); then echo -en "|$RED"; else echo -en "|$GREEN"; fi
  echo -en "$(printf %18s ${sz_bzip2_reduction})$RESET"

  # clean up
  rm ${temp_dir}/${file}."sz"
  rm ${temp_dir}/${file}

  echo "|"
done
