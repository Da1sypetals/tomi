import pickle
import json


def pretty_print_dict(d):
    """
    Pretty prints a dictionary using json.dumps.
    """
    return json.dumps(d, indent=4)


path = "/home/da1sypetals/dev/torch-snapshot/snapshots/snapshot.pickle"
# unpickle and print

with open(path, "rb") as f:
    data = pickle.load(f)


structure = pretty_print_dict(data)

# save to file
with open("pretty_dict.json", "w") as f:
    f.write(structure)
