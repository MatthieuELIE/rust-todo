#!/usr/bin/env bash
set -euo pipefail

mutants=$(cargo mutants --list --no-config)
patterns=$(sed -n "s/^[[:space:]]*'\(.*\)',\$/\1/p" .cargo/mutants.toml)
unmatched=0
positions_re='^(.*):\(([0-9:|]+)\):(.*)$'

anchors() {
    if [[ $1 =~ $positions_re ]]; then
        local prefix=${BASH_REMATCH[1]} rest=${BASH_REMATCH[3]} position
        IFS='|' read -ra positions <<<"${BASH_REMATCH[2]}"
        for position in "${positions[@]}"; do
            echo "$prefix:$position:$rest"
        done

        return
    fi

    echo "$1"
}

if [ -z "$patterns" ]; then
    echo "no exclude_re expression found in .cargo/mutants.toml"
    exit 1
fi

while IFS= read -r anchor; do
    if ! grep -Eq -- "$anchor" <<<"$mutants"; then
        echo "exclude_re matches no mutant: $anchor"
        unmatched=1
    fi
done < <(while IFS= read -r pattern; do anchors "$pattern"; done <<<"$patterns")

exit "$unmatched"
