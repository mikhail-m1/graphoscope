#!/bin/sh

case $1 in
    "dot") 
        dot -Tpng < $2  > /tmp/1.png
        open /tmp/1.png
        ;;
    "my")
        cargo run cmd $2 > /tmp/1.svg || exit 1
        open /tmp/1.svg
        ;;
    *)
        echo "Use '$0 [dot|my] <filename>'"
        ;;
esac
