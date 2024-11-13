# webgraph-ans-rs 

Web graphs are fundamental structures that represent the very complex nature of a specific portion of the World Wide Web 
used in a variety of applications such as search engines. In practical terms, a web graph is a mathematical
abstraction in which there is a node for each web page and an arc from node 𝑥 to node 𝑦 if the page associated with the first node contains a hyperlink to the page associated with
the second. It's easy to guess that the size of these enormous structures makes the traditional way of storing them
obsolete. 

One of the greatest frameworks built with the goal of compressing web graphs is [WebGraph](https://github.com/vigna/webgraph-rs),
a framework that, beyond offering various tools that can be used to operate un such structures, exploits the properties 
of web graphs (locality and similarity), as well as some other ideas tailored for the context, to compress them in an efficient format called BVGraph.
 
This project aims to improve the records of the mentioned frameworks, which have been standing for almost
two decades, by switching from instantaneous codes to [Asymmetrical Numeral Systems](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems) (ANS) when compressing
the graph in the BvGraph format.  

### Compressor
WIP

### Binaries
The binary that can be used to recompress a .graph is bvcomp:
```
$ cargo build --release --bin bvcomp
$ ./target/release/bvcomp <path_to_graph> <output_dir> <new_graph_name> [<compression_params>]
```
For example:
```
$ ./target/release/bvcomp tests/data/cnr-2000/cnr-2000 ans-cnr-2000
```
This command recompresses the cnr-2000.graph file located in the tests/data/cnr-2000/ directory using the default compression parameters and saves the new compressed graph in the current directory as ans-cnr-2000.graph.

Note: <compression_params> is optional. When not specified, default compression values are utilized.

### Benches
WIP

PS: graphs can be found [here](http://law.di.unimi.it/datasets.php).