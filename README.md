# Graphoscope

Graphoscope is Rust library to visualize graphs. It is based on the DOT algorithm from the Graphvis package.
I created it to see how Rust code can be compiled into WebAssembly and run in a browser and at the same time have fun with graph algorithms. 

The long term goals are:
* full dot files support (at least in parser),
* the same or close visualization quality to `dot` from the Graphviz,
* big graphs support (see below),
* interactive graphs, right now a node can be highlighted but in the future it should highlight links as well and modify graph representation.

## Big graphs support

It’s actually the reason why I started this project. I had a dot file with ~1000 nodes connected to one,
and the dot file stuck for a long time and produced something unusable. So I de

You can see the current state here, dotfile parser is basic, and supports just basic syntax.