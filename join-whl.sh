#!/bin/bash -eu

script_dir=$(realpath "$(dirname "${0}")")

join() {
    tmpdir=$(mktemp -d)
    for file in "${@}" ; do
        unzip -o "${file}" -d "${tmpdir}"
    done
    find "${tmpdir}" -name "RECORD" -delete
    cd "${tmpdir}"
    dist_info=$(find . -name '*.dist-info')
    while read -r file ; do
        echo "${file:2},sha256=$(sha256sum "${file}" | awk '{print $1}' | xxd -r -p | base64 | tr +/ -_ | cut -c -43),$(du -b "${file}" | awk '{print $1}')" >> "${dist_info}/RECORD"
    done < <(find ./ -type f)
    echo "serde_mol2-0.1.2.dist-info/RECORD,," >> "${dist_info}/RECORD"
    zip -r "${script_dir}/dist/$(basename "${2}")" -- *

    rm -rf "${tmpdir}"
    cd "${script_dir}"
}

if [ -d "${script_dir}/dist" ] ; then
    rm -rf "${script_dir}/dist"
else
    mkdir "${script_dir}/dist"
fi

while read -r bin_pkg ; do
    while read -r lib_pkg ; do
        join "${bin_pkg}" "${lib_pkg}"
    done < <(find target/wheels/ -name '*cp3*')
done < <(find target/wheels/ -name '*py3*none*')
