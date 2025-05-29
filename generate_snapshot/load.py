# decode_and_debug.py

import argparse
import logging
import pdb
import sys

# Assuming 'encode.py' contains SnapshotDecoder and the Snapshot TypedDict
# and is in the same directory or accessible in PYTHONPATH.
try:
    from snapshot.deprecated.encoding import SnapshotDecoder, Snapshot
except ImportError:
    print("Error: Could not import SnapshotDecoder or Snapshot from 'encode.py'.")
    print("Please ensure 'encode.py' is in the current directory or your PYTHONPATH.")
    sys.exit(1)


# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def main_decode_debug(snapshot_filepath: str):
    logger.info(f"Attempting to decode snapshot file: {snapshot_filepath}")

    # Variables to inspect in PDB
    decoded_snapshot: Snapshot | None = None
    error_during_decoding: Exception | None = None

    try:
        with open(snapshot_filepath, "rb") as f:
            encoded_bytes = f.read()

        logger.info(
            f"Read {len(encoded_bytes)} bytes from the file '{snapshot_filepath}'."
        )

        decoder = SnapshotDecoder(encoded_bytes)
        decoded_snapshot = decoder.decode()  # This should return a Snapshot TypedDict

        logger.info("Snapshot decoded successfully.")
        if decoded_snapshot:
            # Accessing TypedDict keys directly after successful decode
            logger.info(f"  Number of segments: {len(decoded_snapshot['segments'])}")
            logger.info(
                f"  Number of device trace lists: {len(decoded_snapshot['device_traces'])}"
            )

            if decoded_snapshot["segments"]:
                logger.info(
                    f"    First segment total size: {decoded_snapshot['segments'][0]['total_size']}"
                )
                logger.info(
                    f"    Number of blocks in first segment: {len(decoded_snapshot['segments'][0]['blocks'])}"
                )
            if (
                decoded_snapshot["device_traces"]
                and decoded_snapshot["device_traces"][0]
            ):
                logger.info(
                    f"    Number of entries in first device trace: {len(decoded_snapshot['device_traces'][0])}"
                )

    except FileNotFoundError:
        errMsg = f"Error: Snapshot file not found at '{snapshot_filepath}'"
        logger.error(errMsg)
        error_during_decoding = FileNotFoundError(errMsg)
    except Exception as e:
        logger.error(f"An error occurred during decoding: {e}", exc_info=True)
        error_during_decoding = e

    print("\n--- Entering PDB Session ---")
    if decoded_snapshot:
        print("The 'decoded_snapshot' variable contains the decoded data.")
    if error_during_decoding:
        print(
            f"An error occurred: 'error_during_decoding' contains the exception ({type(error_during_decoding).__name__})."
        )
    print("Type 'c' or 'continue' to exit PDB and the script.")

    pdb.set_trace()

    # Execution will pause here until you exit PDB (e.g., by typing 'c' or 'continue')
    logger.info("Exited PDB session.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Decode a PyTorch memory snapshot file and enter PDB for debugging."
    )
    parser.add_argument(
        "filepath",
        type=str,
        nargs="?",  # Makes the argument optional
        default="memory_snapshot_example_encoded.snap",  # Default filename
        help="Path to the encoded snapshot (.snap) file. "
        "Defaults to 'memory_snapshot_example_encoded.snap' if not provided.",
    )
    args = parser.parse_args()

    main_decode_debug(args.filepath)
