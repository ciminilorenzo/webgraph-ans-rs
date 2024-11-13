# webgraph-ans 

In practical terms, a web graph is a mathematical abstraction in which there is a node for each web page 
and an arc from node 𝑥 to node 𝑦 if the page associated to the first node contains a hyperlink to the page associated to
the second. It's easy to guess that the size of these enormous structures makes the traditional way of storing them
not efficient. 

One of the greatest frameworks built with the goal of compressing web graphs is [WebGraph](https://github.com/vigna/webgraph-rs),
a framework that, beyond offering various tools that can be used to operate un such structures, exploits the properties 
of web graphs (locality and similarity), as well as some other ideas tailored for the context, to compress them in an efficient format called BVGraph.
 
This project aims to improve the records of the mentioned frameworks, which have been standing for almost
two decades, by switching from instantaneous codes to [Asymmetrical Numeral Systems](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems) (ANS) when compressing
the graph in the BvGraph format.  

## Quick setup
This crate supplies two different unit structs exposing methods to load a previously ANS-encoded BvGraph or recompress
a BvGraph using the ANS-based approach.

Since this crate is based on [webgraph](https://github.com/vigna/webgraph-rs), please refer to its
docs to learn more about the files and structs cited next.

### Loading a BVGraphSeq with ANSBvGraphSeq
To load a [`BvGraphSeq`], you only need the `BASENAME.ans`file. Then, you can use `ANSBvGraphSeq`:

```ignore
 let graph = ANSBvGraphSeq::load("BASENAME")?;
```

### Loading a BVGraph with ANSBvGraph
To load a [`BvGraph`] the user needs to supply the following files: `BASENAME.ans`, `BASENAME.pointers` and 
`BASENAME.states`.

```ignore
 let graph = ANSBvGraph::load("BASENAME")?;
```

### Recompressing a BvGraph using bvcomp
No matter which approach you use, recompressing a BvGraph will produce the `<graph_name>.ans`, 
`<graph_name>.pointers` and`<graph_name>.states` files in the output directory.

The first approach is using the bvcomp binary: 

1. Compile the bvcomp binary:

```ignore
$ cargo build --release --bin bvcomp
```

2. Run bvcomp to recompress the graph

```ignore
$ ./target/release/bvcomp <path_to_graph> <output_dir> <new_graph_name> [<compression_params>]
```
For example

```ignore
$ ./target/release/bvcomp tests/data/cnr-2000/cnr-2000 ans-cnr-2000
```

This command recompresses the cnr-2000.graph file located in tests/data/cnr-2000/ and saves the output in 
current directory with the name ans-cnr-2000. 

Note: [compression_params] is optional. If omitted, [`default`] values are used.

### Recompressing a BvGraph using ANSBvGraph::store()
ANSBvGraph can be even used to recompress a BvGraph using the ANS-based approach. You just need
to use the method `ANSBvGraph::store()` to indicate where the BvGraph is located and where
the output of the encoding must be located, together with customized compression parameters if
needed.

```ignore
 ANSBvGraph::store(
        basename,
        new_basename,
        compression_window,
        max_ref_count as usize,
        min_interval_length,
 )?;
```


### Results

PS: BV graphs can be found [here](http://law.di.unimi.it/datasets.php).

[`BvGraph`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/random_access/struct.BvGraph.html>
[`BvGraphSeq`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/sequential/struct.BvGraphSeq.html>
[`default`]: <https://docs.rs/webgraph/0.1.4/src/webgraph/cli/mod.rs.html#172-206>