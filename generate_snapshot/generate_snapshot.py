import torch
import torch.nn as nn
import torch.optim as optim
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


# Define a simple model
class SimpleModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear1 = nn.Linear(10, 20)
        self.relu = nn.ReLU()
        self.linear2 = nn.Linear(20, 1)

    def forward(self, x):
        return self.linear2(self.relu(self.linear1(x)))


def run_training_with_snapshot(file_prefix="memory_snapshot_example"):
    if not torch.cuda.is_available():
        logger.error("CUDA is not available. Memory snapshot API requires CUDA.")
        return

    # Define the maximum number of memory events to record
    MAX_NUM_OF_MEM_EVENTS_PER_SNAPSHOT = 100_000

    # Initialize model, loss, and optimizer
    model = SimpleModel().cuda()
    loss_fn = nn.MSELoss()
    optimizer = optim.SGD(model.parameters(), lr=0.01)

    # Dummy data
    inputs = torch.randn(64, 10).cuda()
    labels = torch.randn(64, 1).cuda()

    logger.info("Starting memory history recording...")
    # Start recording memory snapshot history
    try:
        torch.cuda.memory._record_memory_history(max_entries=MAX_NUM_OF_MEM_EVENTS_PER_SNAPSHOT)
        logger.info(f"Memory history recording started with max_entries={MAX_NUM_OF_MEM_EVENTS_PER_SNAPSHOT}")
    except Exception as e:
        logger.error(f"Failed to start memory history recording: {e}")
        return

    # Run your PyTorch Model for 5 iterations
    logger.info("Running training loop for 5 iterations...")
    for i in range(17):
        pred = model(inputs)
        loss = loss_fn(pred, labels)
        loss.backward()
        optimizer.step()
        optimizer.zero_grad(set_to_none=True)
        logger.info(f"Iteration {i + 1}/5, Loss: {loss.item():.4f}")

    logger.info("Attempting to capture and process memory snapshot...")
    try:
        # Get the snapshot data object
        # Note: _snapshot() returns the snapshot data; _snapshot(filename) saves to pickle and returns None.
        torch.cuda.memory._dump_snapshot("snapshots/snapshot.pickle")

    except Exception as e:
        logger.error(f"Failed to capture, encode, or save memory snapshot: {e}")
    finally:
        logger.info("Stopping memory history recording...")
        # Stop recording memory snapshot history.
        try:
            torch.cuda.memory._record_memory_history(enabled=None)
            logger.info("Memory history recording stopped.")
        except Exception as e:
            logger.error(f"Failed to stop memory history recording: {e}")


if __name__ == "__main__":
    run_training_with_snapshot()
