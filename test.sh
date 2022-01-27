#!/bin/bash -eu

ok=
error=
exit_handler() {
    if [ -z "${ok}" ]; then
        echo "${error}"
        exit 1
    fi
}
trap exit_handler EXIT

for binary in ./test.py ./target/release/serde-mol2 ; do
    error="(${binary}) Failed simple reading mol2 to a db"
    "${binary}" -i example.mol2 -s db-py-simple.sqlite
    error="(${binary}) Failed simple reading db into mol2"
    "${binary}" -o out.mol2 -s db-py-simple.sqlite
    [ "$(grep -c MOLECULE out.mol2)" == "$(grep -c MOLECULE example.mol2)" ]
    error="(${binary}) Failed adding mol2 to a db with a desc field"
    "${binary}" -i example.mol2 -s db-py-desc.sqlite --desc example
    error="(${binary}) Failed pulling mol2 from a db filtering based on desc; both match and no match should be tested"
    "${binary}" -o out.mol2 -s db-py-desc.sqlite --desc example
    [ "$(grep -c MOLECULE out.mol2)" == "$(grep -c MOLECULE example.mol2)" ]
    "${binary}" -o out.mol2 -s db-py-desc.sqlite --desc something
    [ "$(grep -c MOLECULE out.mol2)" == 0 ]
    error="(${binary}) Failed adding mol2 to a db with a comment field"
    "${binary}" -i example.mol2 -s db-py-comment.sqlite --comment example_comment
    error="(${binary}) Failed pulling mol2 with an added comment and check if the comment is there"
    "${binary}" -o out.mol2 -s db-py-comment.sqlite
    grep -q example_comment out.mol2
    error="(${binary}) Failed pulling mol2 from a db filtering based on comment; both match and no match should be tested"
    "${binary}" -o out.mol2 -s db-py-comment.sqlite --comment example
    [ "$(grep -c MOLECULE out.mol2)" == "$(grep -c MOLECULE example.mol2)" ]
    "${binary}" -o out.mol2 -s db-py-comment.sqlite --comment something
    [ "$(grep -c MOLECULE out.mol2)" == 0 ]
    error="(${binary}) Failed listing descs"
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc1
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc1
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc1
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc2
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc2
    "${binary}" -i example.mol2 -s db-py-descs.sqlite --desc desc3
    [ "$("${binary}" -s db-py-descs.sqlite --list-desc | wc -l)" == 3 ]
    error="(${binary}) limit functionality failed"
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc1
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc1
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc1
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc2
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc2
    "${binary}" -i example.mol2 -s db-py-limits.sqlite --comment desc3
    "${binary}" -o out.mol2 -s db-py-limits.sqlite --limit 2
    [ "$(grep -c MOLECULE out.mol2)" == 2 ]
    error="(${binary}) limit+offset functionality failed"
    "${binary}" -o out.mol2 -s db-py-limits.sqlite --limit 2 --offset 2
    [ "$(grep -c MOLECULE out.mol2)" == 2 ]
    grep -q desc1 out.mol2
    grep -q desc2 out.mol2

    rm -- *.sqlite
    rm out.mol2
done

ok=1
