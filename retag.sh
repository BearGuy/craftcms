#!/bin/bash

# Usage: ./retag.sh <version>
# Example: ./retag.sh 0.2.0

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION=$1

echo "This will delete and recreate v$VERSION tag. Continue? [y/N]"
read -r answer
if [ "$answer" != "y" ]; then
    echo "Aborted"
    exit 1
fi

# Delete local and remote tags
git tag -d "v$VERSION" 2>/dev/null
git push --delete origin "v$VERSION" 2>/dev/null

# Create and push new tag
git tag -a "v$VERSION" -m "Version $VERSION"
git push
git push --tags

echo "Tag v$VERSION has been recreated"
