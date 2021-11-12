#!/bin/bash

echo "Bumping versions to $1"

git grep -l $(cat VERSION) ':!*.lock' ':!CHANGELOG.md' | xargs sed -i '' -e "s/$(cat VERSION)/$1/g"
echo $1 > VERSION
