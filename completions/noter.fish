set -l _noter_commands add compact edit help init remove sync
set -l _noter_edit_commands edit remove

function _noter_note_ids
    noter 100 | grep -Eo '(\([A-Za-z0-9]{10}\))' | tr -d '()'
end

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a add \
    -d "opens a vim editor to create a new node"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a compact \
    -d "remove staled versions and edits"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a edit \
    -d "edit a note (needs a hash as an argument)"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a help \
    -d "Print this message or the help of the given subcommand(s)"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a init \
    -d "initialize folders and directories for noter"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a remove \
    -d "remove a note (needs a hash as an argument)"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a sync \
    -d "sync with remote file"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -a tags \
    -d "show present tags in notes"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -l tag \
    -a "(noter tags)" \
    -d "filter notes by given tag"

complete -f -c noter \
    -n "not __fish_seen_subcommand_from $_noter_commands" \
    -l no-colors \
    -d "show notes without colorizing"

complete -f -c noter \
    -n "__fish_seen_subcommand_from $_noter_edit_commands" \
    -a "(_noter_note_ids)"
