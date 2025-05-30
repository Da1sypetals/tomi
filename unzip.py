import zipfile
import io


def unzip_json_files_in_memory(zip_file_path):
    allocations, elements = None, None
    try:
        with open(zip_file_path, "rb") as f:
            in_memory_zip = io.BytesIO(f.read())

        with zipfile.ZipFile(in_memory_zip, "r") as zf:
            for file_info in zf.infolist():
                if file_info.filename.endswith(".json"):
                    if "allocations" in file_info.filename:
                        with zf.open(file_info.filename, "r") as json_file:
                            content = json_file.read().decode("utf-8")
                        allocations = content
                    elif "elements" in file_info.filename:
                        with zf.open(file_info.filename, "r") as json_file:
                            content = json_file.read().decode("utf-8")
                        elements = content
                    else:
                        # Not our target file, pass
                        pass

    except FileNotFoundError:
        print(f"Error: The file '{zip_file_path}' was not found.")
    except zipfile.BadZipFile:
        print(f"Error: '{zip_file_path}' is not a valid zip file.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

    return allocations, elements


if __name__ == "__main__":
    my_zip_file_path = "/home/da1sypetals/dev/torch-snapshot/snapshots/large/transformer.zip"

    alloc, elem = unzip_json_files_in_memory(my_zip_file_path)

    breakpoint()
