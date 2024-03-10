import os
import subprocess
import sys
import csv
import re

# Names of the graphs to be compressed
ans_graphs = ["cnr-2000"]  # !!!! Modify this array by adding/removing graphs' name as pleased !!!!

# The parameters to be used for the high compressed graphs
high_compressed_params = {"w": "16", "c": "2000000000"}
# The first parameter must be the path to the directory containing the whole set of ans_graphs
graphs_dir = sys.argv[1]
# The second parameter must be the path where the new graph will be stored.
compressed_graphs_dir = sys.argv[2]
# The third parameter must be the path the directory containing webgraph-rs
webgraph_rs_dir = sys.argv[3]

# Check all needed directories are present before actually starting the script.
if not os.path.isdir(graphs_dir):
    print(f"{graphs_dir} it not a directory.\nUsage is: python script.py <graphs' dir> <new graphs' dir> <webgraph-rs "
          f"dir>")
    exit(1)

if not os.path.isdir(webgraph_rs_dir):
    print(f"{webgraph_rs_dir} is not a directory.\nUsage is: python script.py <graphs' dir> <new graphs' dir> "
          f"<webgraph-rs dir>")
    exit(1)

if not os.path.isdir(compressed_graphs_dir):
    print(f"{compressed_graphs_dir} is not a directory.\nUsage is: python script.py <graphs' dir> <new graphs' dir> "
          f"<webgraph-rs dir>")
    exit(1)

# Check all needed files are present before actually starting the script.
for graph in ans_graphs:
    if not os.path.isfile(f"{graphs_dir}{graph}/{graph}.properties") or \
            not os.path.isfile(f"{graphs_dir}{graph}/{graph}-hc.properties") or \
            not os.path.isfile(f"{graphs_dir}{graph}/{graph}.obl") or \
            not os.path.isfile(f"{graphs_dir}{graph}/{graph}.graph") or \
            not os.path.isfile(f"{graphs_dir}{graph}/{graph}-hc.graph"):
        print(f"{graph} is missing some files.")
        print(f"Be sure that in {graphs_dir}{graph} there are the following files:")
        print(f"{graph}.properties, {graph}-hc.properties, {graph}.obl, {graph}.graph", f"{graph}-hc.graph")
        exit(1)

# Build bins
subprocess.run(["cargo", "build", "--release", "--bin", "bvcomp"])
subprocess.run(["cargo", "build", "--release", "--bin", "random_access_bvtest"])
subprocess.run(["cargo", "build", "--release", "--bin", "seq_access_bvtest"])
subprocess.run(["cargo", "build", "--release", "--manifest-path", f"{webgraph_rs_dir}/Cargo.toml"])

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
# The speed of the random access test on the compressed graph
random_access_speed = []
# The speed of the sequential access test on the high compressed graph
sequential_access_speed = []

for graph in ans_graphs:
    # Grep on the .properties file to get the number of arcs
    command = f"grep -w arcs {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
    number_of_arcs = int(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8'))
    arcs.append(number_of_arcs)

    # Grep on the .properties file to get the number of bits per link
    command = f"grep -w bitsperlink {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
    bit_per_link = float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("/n"))
    bv_graphs_size.append(int((bit_per_link * number_of_arcs) / 8))

    # Grep on the .properties file of the .hc graph to get the number of bits per link
    command = f"grep -w bitsperlink {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
    hc_bit_per_link = float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("/n"))
    bv_hc_graphs_size.append(int((hc_bit_per_link * number_of_arcs) / 8))

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

    states = os.path.getsize(f"{compressed_graphs_dir}{graph}.states")
    pointers = os.path.getsize(f"{compressed_graphs_dir}{graph}.pointers")
    ans_phases_sizes.append(states + pointers)

    # The sequential speed test is performed by running seq_access_bvtest on the high compressed graph.
    print(f"Starting sequential speed test on {graph}")
    sequential_speed = (subprocess.run([
        "./target/release/seq_access_bvtest",
        f"{compressed_graphs_dir}{graph}-hc",
    ], stdout=subprocess.PIPE))

    sequential_speed = sorted(
        [float(speed) for speed in sequential_speed.stdout.decode('utf-8').split("\n") if speed != ''])
    sequential_access_speed.append(sequential_speed[len(sequential_speed) // 2])

    # The random speed test is performed by running random_access_bvtest on the compressed graph.
    print(f"Starting random speed test on {graph}")
    random_speed = (subprocess.run([
        "./target/release/random_access_bvtest",
        f"{compressed_graphs_dir}{graph}",
    ], stdout=subprocess.PIPE))

    random_speed = sorted([float(speed) for speed in random_speed.stdout.decode('utf-8').split("\n") if speed != ''])
    random_access_speed.append(random_speed[len(random_speed) // 2])

with open('results.csv', 'w', encoding='UTF8', newline='') as f:
    writer = csv.writer(f)
    # write the header
    writer.writerow([
        'name',
        'BVGraph',
        'ANSBVGraph',
        'bit/link',
        'HC-BVGraph',
        'HC-ANSBVGraph',
        'HC-bit/link',
        'phases',
        'total',
        'random speed (ns/arc)',
        'sequential speed (ns/arc)'
    ])

    for index in range(len(ans_graphs)):
        # if graph.ef does not exist, build it
        if not os.path.isfile(f"{graphs_dir}{ans_graphs[index]}/{ans_graphs[index]}.ef"):
            print("Building the .ef file")
            command = f"{webgraph_rs_dir}target/release/webgraph build ef {graphs_dir}{ans_graphs[index]}/{ans_graphs[index]}"
            subprocess.run(command, shell=True)

        # The random speed test is performed by running random_access_bvtest on the compressed graph.
        print(f"Starting random speed test of {ans_graphs[index]} with webgraph-rs")
        command = f"{webgraph_rs_dir}target/release/webgraph bench bvgraph {graphs_dir}{ans_graphs[index]}/{ans_graphs[index]} --random 10000000"
        lines = subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8')
        speeds = sorted([float((re.split(' +', line))[1]) for line in lines.split("\n")[:-1]])
        # Get the median of the speeds
        bv_random_speed = speeds[len(speeds) // 2]

        # print(f"Starting sequential speed test of {ans_graphs[index]}-hc with webgraph-rs")
        # command = f"{webgraph_rs_dir}target/release/webgraph bench bvgraph {graphs_dir}{ans_graphs[index]}/{ans_graphs[index]}-hc"
        # lines = subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8')
        # speeds = sorted([float((re.split(' +', line))[1]) for line in lines.split("\n")[:-1]])
        # Get the median of the speeds
        # bv_seq_speed = speeds[len(speeds) // 2]

        # bit/link .ans
        bit_link = "{:.3f}".format((ans_sizes[index] * 8) / arcs[index])
        # How much we store w.r.t the .graph (if negative, we are using less space)
        occupation = "{:.0f}%".format(-(((bv_graphs_size[index] - ans_sizes[index]) / bv_graphs_size[index]) * 100))
        # bit/link -hc.ans
        hc_bit_link = "{:.3f}".format((ans_hc_sizes[index] * 8) / arcs[index])
        # How much we save in space w.r.t the -hc.graph
        occupation_hc = "{:.0f}%".format(
            -(((bv_hc_graphs_size[index] - ans_hc_sizes[index]) / bv_hc_graphs_size[index]) * 100))
        # .obl size
        obl_size = os.path.getsize(f"{graphs_dir}{ans_graphs[index]}/{ans_graphs[index]}.obl")
        # How much we spend w.r.t the .obl
        occupation_phases = "{:.0f}%".format(-(((obl_size - ans_phases_sizes[index]) / obl_size) * 100))
        # Random speed comparison
        random_speed_comparison = "{:.1f}%".format(
            -(((bv_random_speed - random_access_speed[index]) / bv_random_speed) * 100))
        # Sequential speed comparison
        # sequential_speed_comparison = "{:.1f}%".format(
        #    -(((bv_seq_speed - sequential_access_speed[index]) / bv_seq_speed) * 100))

        data = [
            ans_graphs[index],
            "{} B".format(bv_graphs_size[index]),
            "{} B".format(ans_sizes[index]),
            "{} ({})".format(bit_link, occupation),
            "{} B".format(bv_hc_graphs_size[index]),
            "{} B".format(ans_hc_sizes[index]),
            "{} ({})".format(hc_bit_link, occupation_hc),
            "{} B({})".format(ans_phases_sizes[index], occupation_phases),
            "{} B".format(ans_sizes[index] + ans_phases_sizes[index]),
            "{} ({})".format(random_access_speed[index], random_speed_comparison),
            "{:.1f}".format(sequential_access_speed[index])
        ]
        writer.writerow(data)

    print("Saving results in ./results.csv")
