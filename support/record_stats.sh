#!/bin/bash
set -eu
set -o pipefail
IFS=$'\n\t'

stats="{\"version_rust\":\"$($RUSTC --version | head -n 1)\","
stats="$stats \"version_zinc\":\"$(git rev-parse HEAD)\","
stats="$stats \"version_date\":\"$(date)\","

lines=$(arm-none-eabi-size build/*.elf | tail -n +2)
IFS=$'\n'
for line in ${lines[@]}; do
  IFS=$'\n\t '
  read -ra stat <<< "$line"
  file=${stat[5]}
  file=${file#build/app_}
  file=${file%\.elf}
  stats="$stats \"$file\":\"${stat[0]}-${stat[1]}-${stat[2]}\","
done
IFS=$'\n\t'

stats="$stats \"version_platform\":\"$PLATFORM\"},"

echo "Submitting statistics"
git clone -b $PLATFORM git@github.com:bharr/zinc-stats
cd zinc-stats
sed -i '$ d' stats.json
echo "$stats" >> stats.json
echo "{}]" >> stats.json
rm -f *.lst *.map
cp ../build/*.lst ../build/*.map .
git add -A .
git commit -m "Updated build $(date)"
git push origin +$PLATFORM
