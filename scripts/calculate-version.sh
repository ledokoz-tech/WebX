#!/usr/bin/env bash
# Calculate version from git commit count

set -e

# Get total commit count
COMMIT_COUNT=$(git rev-list --count HEAD)

# Calculate version components
MAJOR=$((COMMIT_COUNT / 10000))
REMAINING=$((COMMIT_COUNT % 10000))
MINOR=$((REMAINING / 1000))
REMAINING2=$((REMAINING % 1000))
PATCH=$((REMAINING2 / 100))
BUILD=$((REMAINING2 % 100))

# Format version string
if [ $MAJOR -gt 0 ]; then
    VERSION="${MAJOR}.${MINOR}.${PATCH}"
elif [ $MINOR -gt 0 ]; then
    VERSION="0.${MINOR}.${PATCH}"
elif [ $PATCH -gt 0 ]; then
    VERSION="0.0.${PATCH}"
else
    VERSION="0.0.0${BUILD}"
fi

# Add v prefix for releases >= 1.0.0
if [ $MAJOR -ge 1 ]; then
    VERSION="v${VERSION}"
fi

echo "$VERSION"