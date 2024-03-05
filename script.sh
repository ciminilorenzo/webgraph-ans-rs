#!/bin/bash

# Stops the whole script if bvcomp fails compressing a graph.
set -e

declare -a bv_graphs=(1258291)
declare -a ans_graphs=(cnr-2000)
declare -a ans_sizes=()
declare -a arcs=(3216152)


# The first parameter must be the path to the directory containing the whole set of ans_graphs
GRAPHS_DIRECTORY=$1
# The second parameter must be the path where the new graph will be located.
NEW_GRAPH_PATH=$2

if [ ! -d "$GRAPHS_DIRECTORY" ]; then
  echo "$GRAPHS_DIRECTORY doesn't exist."
  exit 1
fi

cargo build --release --bin bvcomp

for graph in "${ans_graphs[@]}"
do
	echo "Starting compression of $graph"
	./target/release/bvcomp $GRAPHS_DIRECTORY$graph $NEW_GRAPH_PATH$graph  

	actual_size=$(wc -c <"$NEW_GRAPH_PATH$graph.ans")
	ans_sizes+=${actual_size}
done

# Print the header of the table
printf "%-15s %-15s %-15s %-15s %-15s\n" "Graph" "BVGraph(bytes)" " ANSBVGraph(bytes)" "Bit/link" "Occupation" > output.csv

for index in {0..0}
do
	bit_link=$(echo "scale=3; ${ans_sizes[$index]}*8 / ${arcs[$index]}" | bc)
	occupation=$(echo "scale=2;((${bv_graphs[$index]} - ${ans_sizes[$index]})/${bv_graphs[$index]})*100" | bc)

	printf "%-15s %-15s %-20s %-15s %-15s\n" "${ans_graphs[$index]}" "${bv_graphs[$index]}" "${ans_sizes[$index]}" "$bit_link" "-$occupation%" >> output.csv
done



