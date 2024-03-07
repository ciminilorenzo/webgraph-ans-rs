import os
import subprocess
import sys
import csv

highly_compressed_params = {"w": "16", "c": "2000000000"}
ans_graphs = ['gsh-2015-host']
# The size, in bytes, of the compressed graphs (.graph)
bv_graphs_size = [1503238553]
# The number of arcs in the graph
arcs = [1802747600]
# The size, in bytes, of the high compressed graphs (-hc.graph)
bv_hc_graphs_size = [1395864371]
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
        # bit/link .ans
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        # bit/link -hc.ans
        hc_bit_link = "{:.3f}".format((ans_hc_sizes[index] * 8) / arcs[index])
        # How much we save in space w.r.t the .graph
        occupation = "-{:.0f}%".format((bv_graphs_size[index] - ans_sizes[index]) / bv_graphs_size[index] * 100)
        # How much we save in space w.r.t the -hc.graph
        occupation_hc = "-{:.0f}%".format((bv_hc_graphs_size[index] - ans_hc_sizes[index]) / bv_hc_graphs_size[index] * 100)
        # bit/link .graph
        bv_graphs_bit_link = ans_sizes[index] * 8 / arcs[index]
        # bit/link -hc.graph
        bv_hc_graphs_bit_link = ans_hc_sizes[index] * 8 / arcs[index]

        data = [
            ans_graphs[index],
            "{} B".format(bv_graphs_size[index]),
            "{} B".format(ans_sizes[index]),
            bv_graphs_bit_link,
            bit_link,
            occupation,
            "{} B".format(bv_hc_graphs_size[index]),
            "{} B".format(ans_hc_sizes[index]),
            bv_hc_graphs_bit_link,
            hc_bit_link,
            occupation_hc,
            "{} B".format(ans_phases_sizes[index]),
            random_access_speed[index],
            sequential_access_speed[index],
        ]

        writer.writerow(data)

print("Saving results in ./results.csv")
