# Graphoscope

Graphoscope is Rust library to visualize graphs. It is based on [the DOT algorithm](https://graphviz.org/documentation/TSE93.pdf) from the Graphvis package.
I created it to see how Rust code can be compiled into WebAssembly and run in a browser and at the same time have fun with graph algorithms. 

The long term goals are:
* full [dot files](https://graphviz.org/doc/info/lang.html) support (at least in parser),
* the same or close visualization quality to [`dot`](https://graphviz.org/pdf/dotguide.pdf) from the [Graphviz](https://graphviz.org/),
* improve big graphs support.

## Big graphs support

It’s actually the reason why I started this project. I had a dot file with around 1000 nodes connected to one, and the dot from graphvis stuck for a long time and produced something unusable. So I decided to make something to work with those graphs.

You can see the current version [here](https://mikhail-m1.github.io/graphoscope/), dotfile parser is primitive, and supports just basic syntax.