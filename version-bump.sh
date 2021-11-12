#!/bin/bash

[ $# -eq 1 ] || exit

echo "Bumping versions to $1"

git grep -l $(cat VERSION) ':!*.lock' ':!CHANGELOG.md' | xargs sed -i '' -e "s/$(cat VERSION)/$1/g"

# Insert version number into CHANGELOG.md
sed -i '' -e "s/## [Unreleased]/## [Unreleased]\n\n[$1] - $(date '+%Y-%m-%d')/" CHANGELOG.md

echo $1 > VERSION
