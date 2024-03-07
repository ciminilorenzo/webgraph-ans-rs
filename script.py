import os
import subprocess
import sys
import csv

high_compressed_params = {"w": "16", "c": "2000000000"}
# Names of the graphs to be compressed
ans_graphs = ["cnr-2000"]
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

# The size, in bytes, of the compressed graphs (.graph)
bv_graphs_size = []
# The number of arcs in the graph
arcs = []
# The size, in bytes, of the high compressed graphs (-hc.graph)
bv_hc_graphs_size = []
# The size, in bytes, of the ans-compressed graph (.phases)
ans_sizes = []
# The size, in bytes, of the ans-compressed hc graph (-hc.ans)
ans_hc_sizes = []
# The size, in bytes, of the phases
ans_phases_sizes = []
random_access_speed = []
sequential_access_speed = []

for graph in ans_graphs:
    number_of_arcs = int(subprocess.run([
        "grep", "-w", "arcs", f"{graphs_dir}{graph}/{graph}.properties", "|", "cut", "-d'='", "-f2"
    ]).stdout.decode('utf-8'))

    arcs.append(number_of_arcs)

    bit_per_link = subprocess.run([
        "grep", "bitsperlink", f"{graphs_dir}{graph}/{graph}.properties", "|", "cut", "-d'='", "-f2"
    ])

    bv_graphs_size.append((int(bit_per_link.stdout.decode('utf-8')) * number_of_arcs) / 8)

    bit_per_link_hc = subprocess.run([
        "grep", "bitsperlink", f"{graphs_dir}{graph}/{graph}-hc.properties", "|", "cut", "-d'='", "-f2"
    ])

    bv_hc_graphs_size.append((int(bit_per_link_hc.stdout.decode('utf-8')) * number_of_arcs) / 8)

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
        "-w", f"{high_compressed_params.get('w')}",
        "-c", f"{high_compressed_params.get('c')}"
        ])

    ans_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}.ans"))
    ans_hc_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}-hc.ans"))
    ans_phases_sizes.append(os.path.getsize(f"{compressed_graphs_dir}{graph}.phases"))

    # The sequential speed test is performed by running seq_access_bvtest on the high compressed graph.
    print(f"Starting sequential speed test of {graph}")
    sequential_speed = (subprocess.run([
        "./target/release/seq_access_bvtest",
        f"{compressed_graphs_dir}{graph}-hc",
    ], stdout=subprocess.PIPE))

    sequential_access_speed.append(sequential_speed.stdout.decode('utf-8'))

    # The random speed test is performed by running random_access_bvtest on the compressed graph.
    print(f"Starting random speed test of {graph}")
    random_speed = (subprocess.run([
        "./target/release/random_access_bvtest",
        f"{compressed_graphs_dir}{graph}",
    ], stdout=subprocess.PIPE))

    random_access_speed.append(random_speed.stdout.decode('utf-8'))

with open('results.csv', 'w', encoding='UTF8', newline='') as f:
    writer = csv.writer(f)
    # write the header
    writer.writerow([
        'name',
        '.graph',
        '.ans',
        'bit/link',
        'hc.graph',
        'hc.ans',
        'hc bit/link',
        'phases',
        'random speed',
        'sequential speed'
    ])

    for index in range(len(ans_graphs)):
        # bit/link .ans
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        # How much we save in space w.r.t the .graph
        occupation = "-{:.0f}%".format((bv_graphs_size[index] - ans_sizes[index]) / bv_graphs_size[index] * 100)
        # bit/link -hc.ans
        hc_bit_link = "{:.3f}".format((ans_hc_sizes[index] * 8) / arcs[index])
        # How much we save in space w.r.t the -hc.graph
        occupation_hc = "-{:.0f}%".format((bv_hc_graphs_size[index] - ans_hc_sizes[index]) / bv_hc_graphs_size[index] * 100)
        # bit/link .graph
        bv_graphs_bit_link = "{:.2f}".format(ans_sizes[index] * 8 / arcs[index])
        # bit/link -hc.graph
        bv_hc_graphs_bit_link = "{:.2f}".format(ans_hc_sizes[index] * 8 / arcs[index])
        # .obl size
        obl_size = os.path.getsize(f"{graphs_dir}{ans_graphs[index]}/{ans_graphs[index]}.obl")
        # How much we spend w.r.t the .obl
        occupation_phases = "{:.0f}%".format((-(obl_size - ans_phases_sizes[index]) / obl_size * 100))

        data = [
            ans_graphs[index],
            "{} B".format(bv_graphs_size[index]),
            "{} B".format(ans_sizes[index]),
            "{}({})".format(bit_link, occupation),
            "{} B".format(bv_hc_graphs_size[index]),
            "{} B".format(ans_hc_sizes[index]),
            "{}({})".format(hc_bit_link, occupation_hc),
            "{}({}) B".format(ans_phases_sizes[index], occupation_phases),
            random_access_speed[index],
            sequential_access_speed[index],
        ]

        writer.writerow(data)

print("Saving results in ./results.csv")
