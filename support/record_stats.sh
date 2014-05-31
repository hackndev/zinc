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

echo "Setting up git"
openssl aes-256-cbc -k $KEY1 -in support/stats-ssh-keys \
 -d -a -out id_travis
chmod 600 id_travis
eval "$(ssh-agent)"
ssh-add ./id_travis
git clone -b $PLATFORM $STATS_REPO zinc-stats

echo "Collecting statistics"
sed -i '$ d' zinc-stats/stats.json
echo "$stats" >> zinc-stats/stats.json
echo "{}]" >> zinc-stats/stats.json

echo "Archiving optimised artefacts"
rm -rf zinc-stats/optimised
mkdir -p zinc-stats/optimised
cp ./build/*.lst ./build/*.map zinc-stats/optimised

#echo "Building unoptimised artefacts"
#rm -rf ./build
#TODO(bharrisau) Waiting on selectable OPT level for Rakefile

#echo "Archiving unoptimised artefacts"
#rm -rf zinc-stats/unoptimised
#mkdir -p zinc-stats/unoptimised
#cp ./build/*.lst ./build/*.map zinc-stats/unoptimised

echo "Submitting statistics"
cd zinc-stats
git config user.email "<build-bot@zinc.rs>"
git config user.name "<zinc> (via TravisCI)"
git add -A .
git commit -m "Updated build $(date)"
git push origin +$PLATFORM
