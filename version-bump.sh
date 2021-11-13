#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Usage $0 VERSION"
    exit 1
fi

echo "Bumping versions to $1"

# GNU/BSD compat
sedi=(-i)
case "$(uname)" in
  # For macOS, use two parameters
  Darwin*) sedi=(-i "")
esac

git grep -l $(cat VERSION) ':!*.lock' ':!CHANGELOG.md' | xargs sed "${sedi[@]}" -e "s/$(cat VERSION)/$1/g"

# Insert version number into CHANGELOG.md
sed "${sedi[@]}" -e "s/## [Unreleased]/## [Unreleased]\n\n[$1] - $(date '+%Y-%m-%d')/" CHANGELOG.md

echo $1 > VERSION

echo "$(cat VERSION) changeset generated, commit and tag"
