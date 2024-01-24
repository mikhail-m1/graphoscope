# TODO

## high priority bug
* repeated links
* check why right and right_explicit return different result, it's in place.rs

## graph
* make graph api consistent, migrate loops to it
* ns: remove length?
* ns: unify shift
* ns: state for negative graph search
* pass layers between functions
* rewrite code for lines in draw
* calculate nodes size base on text, trim 
* same level edges, self edges, back edges, duplicated edges
* implement search for layer with min nodes in ns top bottom 
* debug only asserts

## BUGS:
* same level lines, but it's not possible without minlen attribute

## web
* get samples and add big graph support 
* console for errors and debug output
* nice file element - https://developer.mozilla.org/en-US/docs/Web/API/File/Using_files_from_web_applications
* better style

## graph next
* full dot parser
* implement good edges visualization follow the doc
* collapse/multiply links between same nodes
* ports