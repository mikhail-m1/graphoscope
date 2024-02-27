# TODO
## big graph support in web
* bug fails with graph/src/lib.rs:39:10:need start
* add node search
* check graph when one node connected to all (another generator)

## refactor
* fix graph::add_edge

## web
* rename js functions to camelCase

## high priority bug
* check why right and right_explicit return different result, it's in place.rs

## graph
* make graph api consistent, migrate loops to it
* ns: remove length?
* ns: unify shift
* pass layers between functions
* rewrite code for lines in draw?
* calculate nodes size base on text, trim 
* same level edges, self edges, back edges, duplicated edges
* implement search for layer with min nodes in ns top bottom 
* debug only asserts

## BUGS:
* same level lines, but it's not possible without minlen attribute

## web
* nice file element - https://developer.mozilla.org/en-US/docs/Web/API/File/Using_files_from_web_applications
* better style

## graph next
* add parameter to name svg graph nodes (now `svg_<node_name>`)
* full dot parser
* implement good edges visualization follow the doc
* collapse/multiply links between same nodes
* ports