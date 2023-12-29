# Benches
In benches/ you can find some criterion benchmarks than can be run simply running:
```shell
cargo bench # if you want to run the whole set of benches.
```
otherwise, you can run a specific bench by running:
```shell
cargo bench -- <bench_name>
```
where `<bench_name>` is the name of the bench you want to run. For example:
```shell
cargo bench -- "with 64bit"
```
Like in this example, when the bench is included within a benchmark group (we need to do this in order to make some advanced
configuration), the name of the running benchmark is of the form:
```text
<group_name> <bench_name>
```
In fact, by running the previous
command, we will get:
```text
Decoding with different width for quasi folded/with 64bit
                        time:   [11.356 ms 11.380 ms 11.405 ms]
                        thrpt:  [87.679 Melem/s 87.876 Melem/s 88.058 Melem/s]
```
since the benchmark function "with 64bit" is included in the group "Decoding with different width for quasi folded".

# Examples
In examples/ you can find some examples of the usage of the different compressors. Every source file contains
details about its content. Moreover, the **correspondent binary can be used to profile the example itself**.

- **decoder**: the standard decoder, that is the one that uses 64-bits quasi-folded field for every DecoderModelEntry and the other standard parameters;
- **decoder_32bit**: the decoder that uses all standard parameters but the width of the quasi-folded field, that is 32 bits in this case. 
- **rank9decoder**: a decoder that uses a Rank9Sel bit vector in order to implement the symbol(y) primitive;
- **bitvec_decoder**: a decoder that uses a BitVec in order to handle the folded bits.


# Folded Streaming Range ANS Coder
This library implements a folded streaming range asymmetric numeral system coder. This coder uses a 64-bit state 
and 32 bit renormalization step with some other constrains discussed below.

Our context is characterized by an extreme long sequence of different symbols. For this reason, this implementation uses the
technique called "symbol folding" (from Moffat and Petri's [paper](https://dl.acm.org/doi/10.1145/3397175)) in order to reduce the size of the alphabet.
After the folding process, the symbols are bucketed into buckets of size multiple of `RADIX`, each of one representing
a specific set of symbols.

### Constraints
#### K
According to Moffat and Petri's [paper](https://dl.acm.org/doi/10.1145/3397175) tries, the best value to use is 16 since it generally improves the compression
effectiveness. For this reason, we set it as default:
```text
 I = [M * K ; 2^b * M * K)
 I = [M * 2^4; 2^b * M * 2^4]
```

#### B
This parameter determines how many bits are extracted/inserted from/to the state during the encoding/decoding process. Let's
use a 32-bit normalization.

#### State's allowed interval and M
Having `B` and `K` fixed according to what it's been said, means that the state's interval is going to be:
```
I = [M * K, M * B * k)
  = [M * 2^4, M * 2^32 * 2^4)
  = [M * 2^4, M * 2^36)
```
Since the state is a 64-bit unsigned, it means that `M` can be at most 2^28. 

### Assumptions
This implementation assumes that the most frequent symbols are located in the lower part of the natural numbers.

---
### Encoder

#### Structure


#### Encoding direction
Usually, ANS-based coders decode symbols in reverse order (i.e. LIFO order). To prioritize decoding speed, this implementation 
encodes symbols in reverse order in order to have the decoded sequence in the correct order.
 

---
### Decoder

#### Structure

#### Implementing the symbol(y) primitive
This primitive is implemented in three flavors:
1. Using a vector containing, in each position, the data associated to the symbol equal to the position itself. This solution 
   is the fastest one but implies a lot of overhead;
2. using EliasFano, that used the predecessor primitive in order to find the index associated to correct the symbol;
3. using a Rank9Sel bit vector, that uses the rank primitive in order to find the index associated to the symbol.


## Ideas
1. We can encode some information in the initial state in order to avoid wasting bits;
2. We could store real frequencies instead of the approximated ones since they are likely to be smaller.

   