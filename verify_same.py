import json


def verify_timesteps_offsets_length(json_file_path):
    """
    Verifies if the 'timesteps' and 'offsets' arrays in each object
    of a JSON file have the same length.

    Args:
        json_file_path (str): The path to the JSON file.

    Returns:
        tuple: A tuple containing:
            - bool: True if all objects have 'timesteps' and 'offsets' of the same length,
                    False otherwise.
            - list: A list of dictionaries, where each dictionary contains information
                    about an object with mismatched 'timesteps' and 'offsets' lengths.
                    Each dictionary will have 'index', 'timesteps_length', and 'offsets_length'.
                    Returns an empty list if all lengths match.
    """
    try:
        with open(json_file_path, "r") as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"Error: The file '{json_file_path}' was not found.")
        return False, []
    except json.JSONDecodeError:
        print(f"Error: Could not decode JSON from the file '{json_file_path}'.")
        return False, []

    if not isinstance(data, list):
        print(
            "Warning: The JSON file does not contain a list of objects. "
            "Assuming a single object for verification."
        )
        data = [data]  # Wrap it in a list to handle single object files

    mismatched_objects = []
    all_lengths_match = True

    for i, obj in enumerate(data):
        if not isinstance(obj, dict):
            print(
                f"Warning: Element at index {i} is not a dictionary. Skipping verification for it."
            )
            all_lengths_match = False
            continue

        timesteps = obj.get("timesteps")
        offsets = obj.get("offsets")

        if timesteps is None:
            print(f"Warning: Object at index {i} is missing the 'timesteps' key.")
            all_lengths_match = False
            continue
        if offsets is None:
            print(f"Warning: Object at index {i} is missing the 'offsets' key.")
            all_lengths_match = False
            continue

        if not isinstance(timesteps, list):
            print(f"Warning: 'timesteps' in object at index {i} is not a list.")
            all_lengths_match = False
            continue
        if not isinstance(offsets, list):
            print(f"Warning: 'offsets' in object at index {i} is not a list.")
            all_lengths_match = False
            continue

        if len(timesteps) != len(offsets):
            mismatched_objects.append(
                {
                    "index": i,
                    "timesteps_length": len(timesteps),
                    "offsets_length": len(offsets),
                }
            )
            all_lengths_match = False

    if all_lengths_match:
        print(
            f"All objects in '{json_file_path}' have 'timesteps' and 'offsets' with the same length."
        )
    else:
        print(
            f"Some objects in '{json_file_path}' have 'timesteps' and 'offsets' with different lengths."
        )

    return all_lengths_match, mismatched_objects


if __name__ == "__main__":
    path = "/home/da1sypetals/dev/torch-snapshot/generated/allocations_over_time.json"
    all_lengths_match, mismatched_objects = verify_timesteps_offsets_length(path)
    print(all_lengths_match, mismatched_objects)
    with open(path, "r") as f:
        data = json.load(f)
    print(len(data))
