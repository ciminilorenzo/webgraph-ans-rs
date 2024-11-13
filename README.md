# webgraph-ans 

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

## Quick setup
This crate supplies two different unit structs exposing methods to load a previously ANS-encoded BvGraph or recompress
a BvGraph using the ANS-based approach.

Two unit structures, `ANSBvGraph` and `ANSBvGraphSeq`, are available to load respectively a [`BvGraph`] or
a [`BvGraphSeq`]

### ANSBvGraph
Can be used to load a [`BvGraph`], that is a graph that can be visited both randomly and iteratively. To
correctly load an ANS-encoded graph and retrieve a [`BvGraph`], the user needs to supply the following
files: `BASENAME.ans`, `BASENAME.pointers` and `BASENAME.states`.

```ignore
 let graph = ANSBvGraph::load("BASENAME")?;
```

This struct can be even used to recompress a BvGraph using the ANS-based approach. You just need
to use the method `ANSBvGraph::store()` to indicate where the BvGraph is located and where
the output of the encoding must be located, together with customized compression parameters if 
needed.

We can achieve the same goal by using the binary `bvcomp`

```
$ cargo build --release --bin bvcomp
$ ./target/release/bvcomp <path_to_graph> <output_dir> <new_graph_name> [<compression_params>]
```

For example:
```
$ ./target/release/bvcomp tests/data/cnr-2000/cnr-2000 ans-cnr-2000
```

This command recompresses the cnr-2000.graph file located in the tests/data/cnr-2000/ directory using the default 
compression parameters and stores in the output directory the following files: `ans-cnr-2000.pointers`, 
`ans-cnr-2000.states`and `ans-cnr-2000.ans`.

**Note** <compression_params> is optional. When not specified, default compression values indicated [`here`] are utilized.



### ANSBvGraphSeq
Can be used to load a [`BvGraphSeq`], that is a graph that can be visited iteratively. To
correctly load an ANS-encoded graph and retrieve a [`BvGraphSeq`], the user needs to supply the following
files: `BASENAME.ans`.

```ignore
 let graph = ANSBvGraphSeq::load("BASENAME")?;
```

PS: BV graphs can be found [here](http://law.di.unimi.it/datasets.php).

[`BvGraph`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/random_access/struct.BvGraph.html>
[`BvGraphSeq`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/sequential/struct.BvGraphSeq.html>
[`here`]: <https://docs.rs/webgraph/0.1.4/src/webgraph/cli/mod.rs.html#172-206>