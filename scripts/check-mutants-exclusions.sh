#!/usr/bin/env bash
set -euo pipefail

mutants=$(cargo mutants --list --no-config)
patterns=$(sed -n "s/^[[:space:]]*'\(.*\)',\$/\1/p" .cargo/mutants.toml)
unmatched=0

if [ -z "$patterns" ]; then
    echo "no exclude_re expression found in .cargo/mutants.toml"
    exit 1
fi

while IFS= read -r pattern; do
    if ! grep -Eq -- "$pattern" <<<"$mutants"; then
        echo "exclude_re matches no mutant: $pattern"
        unmatched=1
    fi
done <<<"$patterns"

exit "$unmatched"
