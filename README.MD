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
two decades, by adding another layer of compression by means of 
[Asymmetrical Numeral Systems](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems) (ANS) over the first layer of compression performed by WebGraph.


### Compressor
Since the BVGraph format is composed of 9 models (Outdegree, ReferenceOffset, BlockCount, Blocks, IntervalCount, 
IntervalStart, IntervalLen, FirstResidual, Residual), the model used by the compressor is going to be switched on the 
fly among the nine built for each specific component of the compression format. Moreover, to overcome the problem of dealing with enormous alphabets (we set as maximum symbol 2^48 - 1), the 
symbol folding technique introduced by Moffat and Petri in their [work](https://dl.acm.org/doi/10.1145/3397175) is 
implemented.

In general, the coder uses a 32-bit state and 16-bit renormalization step with some other constraints discussed below.

The compressor's interval for each model is defined as:
```math
 I = [M * K, M * K * B) 
``` 
where:
1. $M$ is the sum of all symbols frequencies (power of two) for a specific model.
2. $K = 2^{16} - M$
3. $B = 2^{16}$

All this is done to guarantee that the shared interval between all models is $[2^16, 2^32)$, a guarantee needed to make the compressor
able to switch models on the fly.

PS: This implementation assumes that the most frequent symbols are the smallest positive integers.

### Binaries
The binary that can be used to recompress a .graph is bvcomp:
```
$ cargo build --release --bin bvcomp
$ ./target/release/bvcomp <path_to_graph> <output_dir> <new_graph_name> <compression_params>
```
For example:
```
$ ./target/release/bvcomp tests/data/cnr-2000/cnr-2000 ans-cnr-2000
```
recompresses with standard compression parameters the cnr-2000.graph file in the tests/data/cnr-2000/ directory and save 
the new compressed graph in the current directory with the name ans-cnr-2000.graph.

PS: graphs can be found [here](http://law.di.unimi.it/datasets.php).