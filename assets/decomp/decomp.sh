#!/bin/sh

set -e;

MODELS=../models
BINARIES=./binaries
DECOMPS=./obj

BINARY=
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(arch) == 'arm64' ]]; then
        echo "detected mac m1";
        BINARY=$BINARIES/coacd.mac-m1
    fi
elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "detected windows";
    BINARY=$BINARIES/coacd.windows
fi

for MODEL in "$MODELS"/*.obj;
do
    BASE=$(basename $MODEL | cut -d. -f1 );
    DECOMP="$DECOMPS/$BASE";
    mkdir -p $DECOMP;
    echo "checking $DECOMP/obj.obj";
    if [ ! -e "$DECOMP"/obj.obj ]; then
        echo "$BASE doesn't exist";
        $BINARY -i $MODEL -o $DECOMP/obj.obj -l $DECOMP/log.txt;
    fi
done

