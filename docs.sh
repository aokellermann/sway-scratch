#!/bin/bash

mkdir -p docs
DESTDIR=docs cargo docs

cd docs || exit 1

pandoc -f man -t html sway-scratch.1 > index.html
pandoc -f man -t html sway-scratch-show.1 > sway-scratch-show.html

sed -i  's/sway-scratch-show(1)/<a href=\"sway-scratch-show\.html\">sway-scratch-show(1)<\/a>/' index.html
