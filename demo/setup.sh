#!/usr/bin/env bash
set -euo pipefail

day() {
    if date -v+0d +%F >/dev/null 2>&1; then
        date -v"${1}d" +%F
    else
        date -d "${1} days" +%F
    fi
}

today=$(day +0)
late=$(day -3)
soon=$(day +4)
later=$(day +12)
created=$(day -9)
finished=$(day -2)

dir=$(mktemp -d)
mkdir "$dir/todo"

cat > "$dir/todo.txt" <<TODO
(A) $created Send the invoices +finance +finance-tax @desk due:$late
(A) $created Call the accountant +finance-tax @phone due:$today
(B) $created Book the venue for the team offsite +offsite @phone due:$soon
(B) $created Review the pull request for the export command +work @computer
(C) $created Draft the agenda for the offsite +offsite @desk due:$later
(C) $created Reply to the designer about the new logo +work @computer wait:designer
$created Write the release notes for the next version, listing every change since the last tag, the breaking ones first, then the fixes and the documentation updates, and thank the people who reported issues +work @computer
$created Buy a birthday present @errands
$created Water the plants @home
$created Renew the gym membership @errands due:$later
$created Fix the squeaky door @home
TODO

cat > "$dir/done.txt" <<DONE
x $finished $created Pay the electricity bill +finance @computer
x $finished $created Prepare the slides for the offsite +offsite @computer
x $finished $created Update the dependencies +work @computer
x $finished $created Sort the receipts +finance-tax @desk
x $finished $created Order the team lunch +offsite @phone
DONE

printf '[theme]\nname = "catppuccin-mocha"\n' > "$dir/todo/config.toml"

echo "export TODO_FILE=$dir/todo.txt"
echo "export XDG_CONFIG_HOME=$dir"
