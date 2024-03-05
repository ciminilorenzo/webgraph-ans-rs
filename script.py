import os
import subprocess
import sys
import csv

# Graphs are: cnr-2000 and in-2004
ans_graphs = ['cnr-2000', 'in-2004']
bv_graphs = [1258291, 4613734]
arcs = [3216152, 16917053]

# The first parameter must be the path to the directory containing the whole set of ans_graphs
graphs_dir = sys.argv[1]
# The second parameter must be the path where the new graph will be stored.
compressed_graphs_dir = sys.argv[2]

if not os.path.isdir(graphs_dir):
    print(f"{graphs_dir} doesn't exist.")
    exit(1)

# Build bvcomp
subprocess.run(["cargo", "build", "--release", "--bin", "bvcomp"])

ans_sizes = []

for graph in ans_graphs:
    print(f"Starting compression of {graph}")
    subprocess.run(["./target/release/bvcomp", f"{graphs_dir}{graph}/{graph}", f"{compressed_graphs_dir}{graph}"])

    with open(f"{compressed_graphs_dir}{graph}.ans", 'rb') as f:
        actual_size = len(f.read())
        ans_sizes.append(actual_size)

with open('results.csv', 'w', encoding='UTF8', newline='') as f:
    # create the csv writer
    writer = csv.writer(f)
    # write the header
    writer.writerow(['name', 'BVGraph(bytes)', 'ANSBVGraph(bytes)', 'bit/link', 'improvement'])

    for index in range(len(ans_graphs)):
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        improvement = "-{:.0f}%".format((bv_graphs[index] - ans_sizes[index]) / bv_graphs[index] * 100)
        data = [ans_graphs[index], bv_graphs[index], ans_sizes[index], bit_link, improvement]

        # write the data
        writer.writerow(data)

print("Saving results in ./results.csv")
