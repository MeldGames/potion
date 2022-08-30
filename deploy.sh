#!/bin/bash
#
# Script for building Rust games for wasm.
# Tested only on Linux, but probably works on Windows too.
# Public Domain or whatever.
#
# Builds crate for wasm.
# Optionally runs it on local server and opens it in browser.
# Optionally uploads build to itch.
# If you don't use itch, ready to upload version should be in "target/wasm_package".
#
# Options:
#     --run [ADDR:PORT]
#         Starts HTTP server at specified address and opens game in a browser.
#         If argument is not specified, default localhost address is used.
#         Press Ctrl+C to stop server and continue script execution.
#
#     --browser <PATH>
#         Set path to the browser used with '--run'.
#         If not specified, usess xdg-open to open default one.
#
#     --http [ADDR:PORT]
#         Same as --run but doesn't open browser
#
#     --itch [USER/PROJECT:CHANNEL]
#         Pushes game to itch.io using butler.
#         If argument is not specified, reads it from "wasm_build.cfg".
#
#         USER is your itch.io username;
#         PROJECT is a project name (as in URL);
#         CHANNEL is a channel name, whatever you want.
#            (see https://itch.io/docs/butler/pushing.html#channel-names)
#         i.e. "username/awesome-game:web-beta"
#
#     --flags <STRING>
#         Passed to "cargo build", unquoted.
#         If not specified, "--release --no-default-features" is used.
#
# Prerequisites:
# * bash, cargo, sed
# * wasm-bindgen (install with "cargo install wasm-bindgen-cli")
# * python3 (for --run option)
# * butler (for --itch option)
#
# Butler configuration:
# * install: https://itch.io/docs/butler/installing.html
#     on Arch it's available as AUR package
# * run "butler login" (needed only once)
# * that's it!
#

set -e  # exit on errors

#
# CONFIG
#

itch_cfg_path=wasm_build.cfg
# dir re-created in the target directory on each build, contains generated files
output_dir=wasm_package

default_run_address=127.0.0.1:8000
default_cargo_flags="--release --no-default-features"

#
# PARSE ARGUMENTS
#

run_address=
run_browser=1
browser=
itch_deploy=
cargo_flags=$default_cargo_flags

while [[ $# -gt 0 ]]; do
case $1 in
    -h|--help)
        echo "Read the description in the script itself"
    exit 1 ;;
    
    --run)
        if [[ -z "$2" ]] || [[ "$2" == -* ]]; then
            run_address=$default_run_address
        else
            run_address=$2
            shift
        fi
    shift ;;

    --browser)
        browser=$2
    shift 2 ;;
    
    --http)
        if [[ -z "$2" ]] || [[ "$2" == -* ]]; then
            run_address=$default_run_address
        else
            run_address=$2
            shift
        fi
        run_browser=0
    shift ;;
    
    --itch)
        if [[ -z "$2" ]] || [[ "$2" == -* ]]; then
            if [[ ! -e "$itch_cfg_path" ]]; then
                echo "ERROR: used --itch option without arguments and $itch_cfg_path doesn't exist"
                exit 1
            fi
            itch_deploy=`cat $itch_cfg_path`
        else
            itch_deploy=$2
            shift
        fi
    shift ;;

    --flags)
        cargo_flags=$2
    shift 2 ;;
    
    *)
        echo "ERROR: unknown script option \"$1\""
    exit 1 ;;
esac
done

#
# BUILD
#

# build
echo "INFO: cargo flags: $cargo_flags"
cargo build $cargo_flags --target wasm32-unknown-unknown

# extract project name from Cargo.toml
#   shamelessly copied from https://github.com/team-plover/warlocks-gambit
project_name="$(cargo metadata --no-deps --format-version 1 |
    sed -n 's/.*"name":"\([^"]*\)".*/\1/p')"
project_version="$(cargo metadata --no-deps --format-version 1 |
    sed -n 's/.*"version":"\([^"]*\)".*/\1/p')"
# extract name of the target directory
target_dir="$(cargo metadata --format-version 1 |
    sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"

project_name="local"
echo $project_name;

# get name of the built file
wasm_file="$target_dir/wasm32-unknown-unknown/release/$project_name.wasm"
if [ ! -e "$wasm_file" ]; then
    echo "ERROR: script is broken, it expects file to exist: $wasm_file"
    exit 1
fi

# find bindgen
#   shamelessly copied from https://github.com/team-plover/warlocks-gambit
BINDGEN_EXEC_PATH="${CARGO_HOME:-$HOME/.cargo}/bin/wasm-bindgen"
if [ ! -e "$BINDGEN_EXEC_PATH" ] ; then
    echo "ERROR: wasm-bindgen not found at \"$BINDGEN_EXEC_PATH\""
    echo "Run \"cargo install wasm-bindgen-cli\" to install it"
    exit 1
fi

# create output directory
output_dir=$target_dir/$output_dir
[ ! -e "$output_dir" ] || rm -r "$output_dir"

# generate js
$BINDGEN_EXEC_PATH --no-typescript \
    --out-dir "$output_dir" --target web "$wasm_file"

# create HTML
html_file="$output_dir/index.html"
cat > $html_file <<EOF
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta content="text/html;charset=utf-8" http-equiv="Content-Type"/>
    </head>
    <body>
        <script type="module">
            import init from './$project_name.js'
            init();
        </script>
    </body>
</html>
EOF

# copy assets
cp -r assets "$output_dir/assets"

echo "INFO: All files required to run wasm build are in "$output_dir""

#
# OPTIONS
#

# run HTTP server & browser
if [[ ! -z "$run_address" ]]; then
    link="http://$run_address/index.html"
    echo "INFO: running HTTP server at \"$run_address\" (page is at \"$link\")"

    addr=`echo $run_address | sed "s/:.*//"`
    port=`echo $run_address | sed "s/.*://"`
    python3 -m http.server --bind $addr --directory "$output_dir" $port &
    server_job=$!

    if [[ "$run_browser" -eq 1 ]]; then
        sleep 2s  # sometimes browser opens faster than server starts ¯\_(ツ)_/¯
        if [[ -z "$browser" ]]; then
            xdg-open "$link"
        else
            "$browser" "$link"
        fi
    fi

    # wait for Ctrl-C; source: https://stackoverflow.com/a/58508884
    ( trap exit SIGINT ; read -r -d '' _ </dev/tty )

    kill $server_job
fi

# deploy to itch
if [[ ! -z "$itch_deploy" ]]; then
    echo "INFO: deploying to itch.io: $itch_deploy"
    butler push --if-changed --userversion=$project_version \
        "$output_dir" "$itch_deploy"
fi

echo "INFO: All files required to run wasm build are in "$output_dir""