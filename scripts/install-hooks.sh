#!/bin/sh
# Points git at scripts/pre-commit. Run once per clone.
set -e
git config core.hooksPath scripts
chmod +x scripts/pre-commit
echo "Hooks installed. core.hooksPath -> scripts"
