# TODO
## big graph support in web
* improve zoom, check touch interface

## bug / improvment
* check why right and right_explicit return different result, it's in place.rs
 looks like it depends from order, just mirror graph if first level node on the right.

## graph
* fix graph::add_edge
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
* rename js functions to camelCase
* nice file element - https://developer.mozilla.org/en-US/docs/Web/API/File/Using_files_from_web_applications
* better style

## graph next
* add parameter to name svg graph nodes (now `svg_<node_name>`)
* full dot parser
* implement good edges visualization follow the doc
* collapse/multiply links between same nodes
* ports