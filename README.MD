[![Actions Status](https://github.com/ciminilorenzo/streaming_rans/workflows/Build%20and%20Test/badge.svg)](https://github.com/ciminilorenzo/streaming_rans/actions)

# StreamingFoldedAns
This library implements a folded streaming range asymmetric numeral system coder. This coder uses a 64-bit state 
and 32 bit renormalization with some other constrains discussed below.

### Constraints
#### K
According to Moffat and Petri's [paper](https://dl.acm.org/doi/10.1145/3397175) tries, the best value to use is 16 since it generally improves the compression
effectiveness. For this reason, we set it as default:
```text
 I = [M * K ; 2^b * M * K)
 I = [M * 2^4; 2^b * M * 2^4]
```

### B
This parameter determines how many bits are extracted/inserted from/to the state during the encoding/decoding process. Let's
use a 32-bit normalization.

### State's allowed interval
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
#### Streaming invariants
In order to be sure that the new state won't fall outside the allowed interval, we will calculate
the upper bound for each symbol (the maximum 'safe' starting point for each symbol). Thus, we
don't need to store the upper bound of the state's allowed interval.
#### Encoding direction
Usually, ANS-based coders decode symbols in reverse order (i.e. LIFO order). To prioritize decoding speed, this implementation 
encode symbols in reverse order and decode in correct input order.

#### Symbol folding
Our context is characterized by an extreme long sequence of different symbols. For this reason, this implementation uses the
technique called "symbol folding" (from Moffat and Petri's [paper](https://dl.acm.org/doi/10.1145/3397175)) in order to reduce the size of the alphabet. 
After the folding process, the symbols are bucketed into buckets of size multiple of `RADIX`, each of one representing 
a specific set of symbols.
 

---
### Decoder
#### Implementing the symbol(y) primitive
This primitive is implemented in three flavors:
1. Using a vector containing, in each position, the data associated to the symbol equal to the position itself. This solution 
   is the fastest one but implies a lot of overhead;
2. using EliasFano, that used the predecessor primitive in order to find the index associated to correct the symbol;
3. using a Rank9Sel bit vector, that uses the rank primitive in order to find the index associated to the symbol.


## Open points
1. We can encode some information in the initial state in order to avoid wasting bits;
2. We could store real frequencies instead of the approximated ones since they are likely to be smaller.

   