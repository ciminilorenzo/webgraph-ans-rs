import os
import subprocess
import sys
import csv

# bvgraph_seq_speed = ["9.5 ns/arc"]
# bvgraph_rand_speed = ["43.8 ns/arc"]
highly_compressed_params = {"w": "16", "c": "2000000000"}
ans_graphs = ['cnr-2000']

bv_graphs_size = [1258291]
bv_hc_graphs_size = [987136]

bv_graphs_bit_link = [3.12]
bv_hc_graphs_bit_link = [2.45]

arcs = [3216152]

# The first parameter must be the path to the directory containing the whole set of ans_graphs
graphs_dir = sys.argv[1]
# The second parameter must be the path where the new graph will be stored.
compressed_graphs_dir = sys.argv[2]

if not os.path.isdir(graphs_dir):
    print(f"{graphs_dir} doesn't exist.")
    exit(1)

# Build bins
subprocess.run(["cargo", "build", "--release", "--bin", "bvcomp"])
subprocess.run(["cargo", "build", "--release", "--bin", "random_access_bvtest"])
subprocess.run(["cargo", "build", "--release", "--bin", "seq_access_bvtest"])

ans_sizes = []
ans_hc_sizes = []
ans_phases_sizes = []
random_access_speed = []
sequential_access_speed = []

for graph in ans_graphs:
    print(f"Starting standard compression of {graph}")
    subprocess.run([
        "./target/release/bvcomp",
        f"{graphs_dir}{graph}/{graph}",
        f"{compressed_graphs_dir}{graph}"
    ])

    print(f"Starting high compression of {graph}")
    subprocess.run([
        "./target/release/bvcomp",
        f"{graphs_dir}{graph}/{graph}",
        f"{compressed_graphs_dir}{graph}-hc",
        "-w", f"{highly_compressed_params.get('w')}",
        "-c", f"{highly_compressed_params.get('c')}"
        ])

    ans_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}.ans"))
    ans_hc_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}-hc.ans"))
    ans_phases_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}.phases"))

    # The sequential speed test is performed by running seq_access_bvtest on the high compressed graph.
    print(f"Starting sequential speed test of {graph}")
    sequential_speed = (subprocess.run([
        "./target/release/seq_access_bvtest",
        f"{compressed_graphs_dir}{graph}-hc",
    ],stdout=subprocess.PIPE))

    sequential_access_speed.append(sequential_speed.stdout.decode('utf-8'))

    # The random speed test is performed by running random_access_bvtest on the compressed graph.
    print(f"Starting random speed test of {graph}")
    random_speed = (subprocess.run([
        "./target/release/random_access_bvtest",
        f"{compressed_graphs_dir}{graph}",
    ],stdout=subprocess.PIPE))

    random_access_speed.append(random_speed.stdout.decode('utf-8'))

with open('results.csv', 'w', encoding='UTF8', newline='') as f:
    # create the csv writer
    writer = csv.writer(f)
    # write the header
    writer.writerow([
        'name',
        '.graph',
        '.ans',
        '.graph bit/link',
        '.ans bit/link',
        'occupation',
        'hc.graph',
        'hc.ans',
        'hc.graph bit/link',
        'hc.ans bit/link',
        'occupation hc',
        'phases',
        'random speed',
        'sequential speed'
    ])

    for index in range(len(ans_graphs)):
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        hc_bit_link = "{:.3f}".format((ans_hc_sizes[index] * 8) / arcs[index])
        occupation = "-{:.0f}%".format((bv_graphs_size[index] - ans_sizes[index]) / bv_graphs_size[index] * 100)
        occupation_hc = "-{:.0f}%".format((bv_hc_graphs_size[index] - ans_hc_sizes[index]) / bv_hc_graphs_size[index] * 100)

        data = [
            ans_graphs[index],
            "{} B".format(bv_graphs_size[index]),
            "{} B".format(ans_sizes[index]),
            bv_graphs_bit_link[index],
            bit_link,
            occupation,
            "{} B".format(bv_hc_graphs_size[index]),
            "{} B".format(ans_hc_sizes[index]),
            bv_hc_graphs_bit_link[index],
            hc_bit_link,
            occupation_hc,
            "{} B".format(ans_phases_sizes[index]),
            random_access_speed[index],
            sequential_access_speed[index],
        ]

        # write the data
        writer.writerow(data)

print("Saving results in ./results.csv")
