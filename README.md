# webgraph-ans 
Very large graphs, such as Web snapshots, large social networks,
or software dependency graphs, can be analyzed using two approaches: distributing 
the computation on multiple computational
units or compressing the graph to fit in the memory of a single
one. A popular instance of the latter approach is the [WebGraph
framework](https://dl.acm.org/doi/10.1145/988672.988752), which has been recently
rethink and reimplemented in [Rust](https://github.com/vigna/webgraph-rs) due to 
some Java limitations such as limited array size ($2^{31}$ elements).

This complementary project aims at enhancing compression performances by replacing
instantaneous codes used by WebGraph with a recently proposed entropy coder,
[Asymmetrical Numeral Systems](https://en.wikipedia.org/wiki/Asymmetric_numeral_systems) (ANS from now on) which, together with other ideas 
such as [symbol folding](https://dl.acm.org/doi/10.1145/3397175), has shown to be 
extremely effective.

## Quick setup
This crate supplies two different unit structs exposing methods to load a previously ANS-encoded BvGraph or recompress
a BvGraph using the ANS-based approach.

Since this crate is based on [WebGraph](https://github.com/vigna/webgraph-rs), please refer to its
docs to learn more about the files and structs cited next.

### Loading a BVGraphSeq with ANSBvGraphSeq
To load a [`BvGraphSeq`], you only need the `BASENAME.ans` file. Then, you can use `ANSBvGraphSeq`:

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
the output of the encoding must be located, together with customized compression parameters.

```ignore
 ANSBvGraph::store(
        basename,
        new_basename,
        compression_window,
        max_ref_count as usize,
        min_interval_length,
 )?;
```

<details>
  <summary> <h2> Results </h2> </summary>

The proposed methodology is evaluated by comparing its compression effectiveness and its decoding speed to the 
performances of WebGraph by using several real web graphs and social graphs available in the [`LAW`] website:

| Graph          | Nodes         | Arcs           |
|----------------|---------------|----------------|
| dblp-2011      | 986,324       | 6,707,236      |
| enwiki-2023    | 6,625,370     | 165,206,104    |
| eu-2015        | 1,070,557,254 | 91,792,261,600 |
| eu-2015-host   | 11,264,052    | 386,915,963    |
| gsh-2015       | 988,490,691   | 33,877,399,152 |
| gsh-2015-host  | 68,660,142    | 1,802,747,600  |
| hollywood-2011 | 2,180,759     | 228,985,632    |
| twitter-2010   | 41,652,230    | 1,468,365,182  |

All experiments were performed on an Intel 12th Gen i7-12700KF CPU equipped with
64GB RAM and 37 MB cache while the code was compiled by using level 3 optimizations
and native CPU instructions.

Ps: the tables present in the next sections are computed using `percomponent_analysis.py` and `script.py`, hence you can
see the performances on your machine running the mentioned scripts.

### Standard compression
The next tables show the comparison between WebGraph and the proposed methodology. In particular, given a graph:
- **BVGraph** is the size of the compressed graph using WebGraph;
- **ANSBVGraph** is the size of the file resulting from the compression of the latter with the proposed methodology; 
- **bit/link** represents the number of bits required to represent an arc of the graph (i.e a link); 
- **phases** indicates the cost of representing all the needed data to randomly access the graph;
- **random speed**  the time needed to enumerate the successors of 10 million random nodes.

The values between parenthesis represent the comparison with WebGraph.


The results highlight how the proposed approach is able to improve the compression performances by an average of 10% on the whole set of 
graphs. This is particularly striking for web graphs, because WebGraph is already using [`specialized codes`] that are 
tailored for the distribution of residuals, and indeed the compression records obtained by BVGraph with LLP ordering
have been standing for almost two decades. This is the first time a significant increase in
compression has been observed. Even social graphs, notably hard to compress due to the absence of the exploitable 
patterns present in web graphs, are compressed with a better outcome.

| Graph          | BVGraph  | ANSBVGraph | bit/link     | phases           | total    | random speed (ns/arc) |
|----------------|----------|------------|--------------|------------------|----------|-----------------------|
| dblp-2011      | 7.0MiB   | 6.2MiB     | 7.728 (-11%) | 4.2MiB (+341%)   | 10.4MiB  | 58.0 (+10.9%)         |
| enwiki-2023    | 266.8MiB | 244.0MiB   | 12.389 (-9%) | 30.4MiB (+263%)  | 274.4MiB | 55.0 (+65.2%)         |
| eu-2015        | 12.8GiB  | 11.4GiB    | 1.064 (-11%) | 4.6GiB (+313%)   | 16.0GiB  | 22.0 (+22.7%)         |
| eu-2015-host   | 151.0MiB | 140.9MiB   | 3.056 (-7%)  | 49.6MiB (+299%)  | 190.6MiB | 23.0 (+37.7%)         |
| gsh-2015       | 8.1GiB   | 7.2GiB     | 1.823 (-11%) | 4.2GiB (+330%)   | 11.4GiB  | 36.0 (+25.3%)         |
| gsh-2015-host  | 1.3GiB   | 1.2GiB     | 5.764 (-7%)  | 306.9MiB (+289%) | 1.5GiB   | 38.0 (+32.0%)         |
| hollywood-2011 | 139.6MiB | 132.0MiB   | 4.834 (-5%)  | 10.2MiB (+245%)  | 142.2MiB | 32.0 (+64.9%)         |
| twitter-2010   | 2.4GiB   | 2.2GiB     | 12.974 (-8%) | 194.1MiB (+237%) | 2.4GiB   | 45.0 (+61.0%)         |





The next table gives a more in-depth look at the compression results shown above. In particular, the compression rates 
of the proposed methodology are shown and compared with those of WebGraph for each individual component of the BVGraph 
format, allowing the performance of compression using ANS to be highlighted without taking into account the
cost associated with storing the models.

| Name           | Blocks           | Intervals         | Residuals        | References        | Outdegree         |
|----------------|------------------|-------------------|------------------|-------------------|-------------------|
| dblp-2011      | 546.2KiB (-2.4%) | 256.1KiB (-33.1%) | 4.6MiB (-11.4%)  | 235.8KiB (+1.1%)  | 489.6KiB (-18.0%) |
| enwiki-2023    | 11.2MiB (-2.5%)  | 1.3MiB (-38.3%)   | 224.6MiB (-8.3%) | 2.1MiB (-5.0%)    | 4.6MiB (-24.7%)   |
| eu-2015        | 1.6GiB (-5.4%)   | 1.1GiB (-22.1%)   | 7.4GiB (-6.8%)   | 334.4MiB (-24.1%) | 948.0MiB (-26.4%) |
| eu-2015-host   | 7.7MiB (+0.5%)   | 2.8MiB (-29.0%)   | 122.4MiB (-6.4%) | 2.2MiB (-5.1%)    | 5.8MiB (-8.0%)    |
| gsh-2015       | 814.7MiB (-4.4%) | 480.9MiB (-18.1%) | 4.9GiB (-8.0%)   | 305.0MiB (-21.8%) | 749.7MiB (-24.7%) |
| gsh-2015-host  | 70.4MiB (+0.1%)  | 23.0MiB (-25.4%)  | 1.1GiB (-7.0%)   | 16.2MiB (-6.0%)   | 39.9MiB (-9.3%)   |
| hollywood-2011 | 10.8MiB (-0.4%)  | 6.5MiB (-26.7%)   | 111.9MiB (-4.0%) | 649.3KiB (-14.1%) | 2.0MiB (-23.2%)   |
| twitter-2010   | 79.3MiB (+6.8%)  | 7.9MiB (-45.7%)   | 2.1GiB (-8.3%)   | 12.0MiB (+1.5%)   | 26.2MiB (-18.7%)  |


### High compression
Table next table shows instead the encoding results of the high-compressed graphs, with an average increase of the 
compression performances of around 10% even in this scenario. Like in the previous results, the sequential speed, which 
measures the time needed to enumerate the successors of all graph nodes visited sequentially, increases in a reasonable
manner allowing the user to retrieve the data at very high speed despite the additional
layer of compression.

| Graph            | hc-BVGraph   | hc-ANSBVGraph | hc-bit/link  | Sequential Speed (ns/arc) |
|------------------|--------------|---------------|--------------|---------------------------|
| dblp-2011        | 6.8MiB       | 6.0MiB        | 7.456 (-12%) | 21.5 (+94.2%)             |
| enwiki-2023      | 259.8MiB     | 236.0MiB      | 11.985 (-9%) | 18.7 (+74.9%)             |
| eu-2015          | 9.6GiB       | 8.6GiB        | 0.801 (-11%) | 3.4 (-13.0%)              |
| eu-2015-host     | 144.5MiB     | 134.5MiB      | 2.916 (-7%)  | 6.4 (+81.6%)              |
| gsh-2015         | 6.3GiB       | 5.6GiB        | 1.418 (-12%) | 4.4 (+46.6%)              |
| gsh-2015-host    | 1.2GiB       | 1.2GiB        | 5.492 (-7%)  | 9.9 (+72.5%)              |
| hollywood-2011   | 133.3MiB     | 125.6MiB      | 4.602 (-6%)  | 10.5 (+71.6%)             |
| twitter-2010     | 2.4GiB       | 2.2GiB        | 12.731 (-8%) | 16.4 (+76.3%)             |

Even if less important, rates of compression of single components achieve peaks of around 45% despite the smaller
presence of exploitable patterns caused by the higher level of compression performed by
WebGraph.

| Graph          | Outdegree         | Reference         | Block            | Intervals         | Residuals        |
|----------------|-------------------|-------------------|------------------|-------------------|------------------|
| dblp-2011      | 489.6KiB (-18.0%) | 250.8KiB (-3.6%)  | 634.9KiB (-0.6%) | 232.1KiB (-35.6%) | 4.3MiB (-12.4%)  |
| enwiki-2023    | 4.6MiB (-24.7%)   | 2.7MiB (-10.6%)   | 14.2MiB (+1.0%)  | 1.0MiB (-43.7%)   | 213.3MiB (-9.1%) |
| eu-2015        | 947.8MiB (-26.4%) | 337.0MiB (-23.9%) | 1.4GiB (-5.7%)   | 457.1MiB (-24.7%) | 5.4GiB (-6.6%)   |
| eu-2015-host   | 5.8MiB (-8.0%)    | 2.4MiB (-1.6%)    | 8.7MiB (+2.7%)   | 2.0MiB (-33.6%)   | 115.5MiB (-7.1%) |
| gsh-2015       | 749.7MiB (-24.7%) | 275.0MiB (-20.4%) | 661.6MiB (-6.2%) | 202.3MiB (-25.0%) | 3.7GiB (-7.9%)   |
| gsh-2015-host  | 39.9MiB (-9.3%)   | 18.8MiB (-5.8%)   | 80.8MiB (+1.0%)  | 15.5MiB (-31.1%)  | 1.0GiB (-7.5%)   |
| hollywood-2011 | 1.9MiB (-23.2%)   | 514.0KiB (-25.3%) | 11.8MiB (-0.7%)  | 5.7MiB (-28.6%)   | 105.5MiB (-4.3%) |
| twitter-2010   | 26.2MiB (-18.7%)  | 13.5MiB (-5.9%)   | 97.8MiB (+9.6%)  | 6.3MiB (-51.3%)   | 2.0GiB (-8.8%)   |

### Conclusions 
The final approach correctly uses ANS as a second step of compression over
the encoding employing the BV format, successfully achieving the goal of improving its
excellent performances while maintaining fast access time. The average increase in
performance is by no means negligible, with an average value of 10% there is a saving in
storage, which with very large web graphs can be significant.
The proposed methodology represents the first and unique attempt to make this encoding
technique work in combination with the BV format, thus it can certainly be enhanced by
improving some of its parts.

An important open problem is that of improving the storage of phases. At this time for
very sparse graphs phases are so large that they partially cancel the improvement in the
size of the bit stream. While this is not important for sequential decoding, an improvement
in phase storage would further reduce the footprint of a random-access ANSBVGraph.


</details>

PS: BV graphs can be found [here](http://law.di.unimi.it/datasets.php).

[`BvGraph`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/random_access/struct.BvGraph.html>
[`BvGraphSeq`]: <https://docs.rs/webgraph/0.1.4/webgraph/graphs/bvgraph/sequential/struct.BvGraphSeq.html>
[`default`]: <https://docs.rs/webgraph/0.1.4/src/webgraph/cli/mod.rs.html#172-206>
[`LAW`]: <https://law.di.unimi.it/>
[`specialized codes`]: <https://vigna.di.unimi.it/ftp/papers/Codes.pdf>