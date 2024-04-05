import os
import subprocess
import sys
import csv


def sizeof_fmt(num, suffix="B"):
    for unit in ("", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"):
        if abs(num) < 1024.0:
            return f"{num:3.1f}{unit}{suffix}"
        num /= 1024.0
    return f"{num:.1f}Yi{suffix}"


ans_graphs = [
    "dblp-2011", "enwiki-2023", "eu-2015", "eu-2015-host",
    "gsh-2015", "gsh-2015-host", "hollywood-2011", "twitter-2010"
]

# The first parameter must be the path to the directory containing the whole set of ans_graphs
graphs_dir = sys.argv[1]
# The second parameter must be the path where the new graph will be stored.
compressed_graphs_dir = sys.argv[2]

for graph in ans_graphs:
    if not os.path.isfile(f"{graphs_dir}{graph}/{graph}.properties"):
        print(f"{graph} is missing .properties in {graphs_dir}{graph}")
        exit(1)

    if not os.path.isfile(f"{compressed_graphs_dir}{graph}.output"):
        print(f"{graph} is missing .output in {compressed_graphs_dir}")
        exit(1)

with open('components_analysis.csv', 'w', encoding='UTF8', newline='') as f:
    writer = csv.writer(f)
    # write the header
    writer.writerow([
        'name',
        'Blocks',
        'Intervals',
        'Residuals',
        'References',
        'Outdegree'
    ])

    for graph in ans_graphs:
        # Get bytes used by bvgraph to represent its components
        command = f"grep ^bitsforblocks {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
        bv_blocks = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforintervals {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
        bv_intervals = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforresiduals {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
        bv_residuals = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforreferences {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
        bv_references = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforoutdegrees {graphs_dir}{graph}/{graph}.properties | cut -d'=' -f2"
        bv_outdegrees = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        # Get bytes used by ANSBVGraph to represent its components
        command = (
            f"grep -wns 'Building the model with EntropyEstimator...' {compressed_graphs_dir}{graph}.output -A 10 | "
            f"cut -d'|' -f5 | cut -d'(' -f1")
        output = subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').split("\n")
        ans_blocks = int(output[4]) + int(output[5])
        ans_intervals = int(output[6]) + int(output[7]) + int(output[8])
        ans_residuals = int(output[9]) + int(output[10])
        ans_references = int(output[3])
        ans_outdegree = int(output[2])

        writer.writerow([
            graph,
            "{} ({:.1f}%)".format(sizeof_fmt(ans_blocks), -(((bv_blocks - ans_blocks) / bv_blocks) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_intervals), -(((bv_intervals - ans_intervals) / bv_intervals) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_residuals), -(((bv_residuals - ans_residuals) / bv_residuals) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_references),
                                  -(((bv_references - ans_references) / bv_references) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_outdegree),
                                  -(((bv_outdegrees - ans_outdegree) / bv_outdegrees) * 100))
        ])

print("Saving results in ./components_analysis.csv")

# Now for hc-graph
with open('components_analysis_hc.csv', 'w', encoding='UTF8', newline='') as f:
    writer = csv.writer(f)
    # write the header
    writer.writerow([
        'name',
        'Blocks',
        'Intervals',
        'Residuals',
        'References',
        'Outdegree'
    ])

    for graph in ans_graphs:
        # Get bytes used by bvgraph to represent its components
        command = f"grep ^bitsforblocks {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
        bv_blocks = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforintervals {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
        bv_intervals = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforresiduals {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
        bv_residuals = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforreferences {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
        bv_references = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        command = f"grep ^bitsforoutdegrees {graphs_dir}{graph}/{graph}-hc.properties | cut -d'=' -f2"
        bv_outdegrees = int(float(subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').strip("\n")) / 8)

        # Get bytes used by ANSBVGraph to represent its components
        command = (
            f"grep -wns 'Building the model with EntropyEstimator...' {compressed_graphs_dir}{graph}-hc.output -A 10 | "
            f"cut -d'|' -f5 | cut -d'(' -f1")
        output = subprocess.run(command, capture_output=True, shell=True).stdout.decode('utf-8').split("\n")
        ans_blocks = int(output[4]) + int(output[5])
        ans_intervals = int(output[6]) + int(output[7]) + int(output[8])
        ans_residuals = int(output[9]) + int(output[10])
        ans_references = int(output[3])
        ans_outdegree = int(output[2])

        writer.writerow([
            graph,
            "{} ({:.1f}%)".format(sizeof_fmt(ans_blocks), -(((bv_blocks - ans_blocks) / bv_blocks) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_intervals), -(((bv_intervals - ans_intervals) / bv_intervals) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_residuals), -(((bv_residuals - ans_residuals) / bv_residuals) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_references),
                                  -(((bv_references - ans_references) / bv_references) * 100)),
            "{} ({:.1f}%)".format(sizeof_fmt(ans_outdegree),
                                  -(((bv_outdegrees - ans_outdegree) / bv_outdegrees) * 100))
        ])

print("Saving results in ./components_analysis_hc.csv")
