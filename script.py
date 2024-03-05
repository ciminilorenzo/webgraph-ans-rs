import os
import subprocess
import sys
import csv
import re

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
# Build bvtest
subprocess.run(["cargo", "build", "--release", "--bin", "bvtest"])

ans_sizes = []
ans_phases_sizes = []
random_access_speed = []
sequential_access_speed = []

for graph in ans_graphs:
    print(f"Starting compression of {graph}")
    subprocess.run(["./target/release/bvcomp", f"{graphs_dir}{graph}/{graph}", f"{compressed_graphs_dir}{graph}"])

    ans_size = os.path.getsize(f"{compressed_graphs_dir}{graph}.ans")
    ans_sizes.append(ans_size)
    phases_size = os.path.getsize(f"{compressed_graphs_dir}{graph}.phases")
    ans_phases_sizes.append(phases_size)

    print(f"Starting random/sequential speed test of {graph}")

    timing = subprocess.run([
        "./target/release/bvtest",
        f"{compressed_graphs_dir}{graph}",
    ],
        stderr=subprocess.PIPE,
    )

    # Define regular expression pattern
    pattern = r'INFO - Elapsed: (\d+\w{1,2}) \['

    # Find all matches
    matches = re.findall(pattern, f"{timing}")

    random = matches[0]
    sequential = matches[1]
    random_access_speed.append(random)
    sequential_access_speed.append(sequential)

with open('results.csv', 'w', encoding='UTF8', newline='') as f:
    # create the csv writer
    writer = csv.writer(f)
    # write the header
    writer.writerow(['name', 'BVGraph', 'ANSBVGraph', 'improvement', 'bit/link', 'phases', 'random', 'sequential'])

    for index in range(len(ans_graphs)):
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        improvement = "-{:.0f}%".format((bv_graphs[index] - ans_sizes[index]) / bv_graphs[index] * 100)
        data = [
            ans_graphs[index],
            "{} B".format(bv_graphs[index]),
            "{} B".format(ans_sizes[index]),
            improvement,
            bit_link,
            "{} B".format(ans_phases_sizes[index]),
            random_access_speed[index],
            sequential_access_speed[index],
        ]

        # write the data
        writer.writerow(data)

print("Saving results in ./results.csv")
