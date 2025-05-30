import torch
import torch.nn as nn
import torch.optim as optim
import math


# 1. Define the Positional Encoding Layer
class PositionalEncoding(nn.Module):
    def __init__(self, d_model, max_len=5000):
        super(PositionalEncoding, self).__init__()
        pe = torch.zeros(max_len, d_model)
        position = torch.arange(0, max_len, dtype=torch.float).unsqueeze(1)
        div_term = torch.exp(torch.arange(0, d_model, 2).float() * (-math.log(10000.0) / d_model))
        pe[:, 0::2] = torch.sin(position * div_term)
        pe[:, 1::2] = torch.cos(position * div_term)
        pe = pe.unsqueeze(0).transpose(0, 1)
        self.register_buffer("pe", pe)

    def forward(self, x):
        return x + self.pe[: x.size(0), :]


# 2. Define the Transformer Model
class TransformerModel(nn.Module):
    def __init__(self, ntoken, d_model, nhead, d_hid, nlayers, dropout=0.5):
        super(TransformerModel, self).__init__()
        self.model_type = "Transformer"
        self.pos_encoder = PositionalEncoding(d_model, max_len=5000)
        self.encoder_layer = nn.TransformerEncoderLayer(d_model, nhead, d_hid, dropout)
        self.transformer_encoder = nn.TransformerEncoder(self.encoder_layer, nlayers)
        self.decoder = nn.Linear(d_model, ntoken)
        self.d_model = d_model
        self.embedding = nn.Embedding(ntoken, d_model)
        self.init_weights()

    def init_weights(self):
        initrange = 0.1
        self.embedding.weight.data.uniform_(-initrange, initrange)
        self.decoder.bias.data.zero_()
        self.decoder.weight.data.uniform_(-initrange, initrange)

    def forward(self, src, src_mask=None):
        src = self.embedding(src) * math.sqrt(self.d_model)
        src = self.pos_encoder(src)
        output = self.transformer_encoder(src, src_mask)
        output = self.decoder(output)
        return output


# 3. Set up Hyperparameters and Mock Data
# Check if CUDA is available
device = torch.device("cuda:0")
print(f"Using device: {device}")

if device.type == "cuda":
    # Clear any previous CUDA memory allocations
    torch.cuda.empty_cache()
    # Reset the peak memory tracker
    torch.cuda.reset_peak_memory_stats()

# Model hyperparameters
ntoken = 20000  # Size of vocabulary
d_model = 512  # Embedding dimension
nhead = 8  # Number of attention heads
d_hid = 2048  # Dimension of the feedforward network model in nn.TransformerEncoderLayer
nlayers = 36  # Number of nn.TransformerEncoderLayer in nn.TransformerEncoder
dropout = 0.5  # Dropout probability

# Training parameters
seq_len = 35  # Sequence length
batch_size = 32  # Batch size
num_iterations = 32  # Number of training iterations

# Create mock data
mock_src = torch.randint(0, ntoken, (seq_len, batch_size)).to(device)
mock_target = torch.randint(0, ntoken, (seq_len * batch_size,)).to(device)

# 4. Initialize Model, Optimizer, and Loss Function
model = TransformerModel(ntoken, d_model, nhead, d_hid, nlayers, dropout).to(device)
criterion = nn.CrossEntropyLoss()
optimizer = optim.Adam(model.parameters(), lr=0.001)

# 5. Run Training Loop with Mock Data
model.train()  # Set the model to training mode

torch.cuda.memory._record_memory_history()
for i in range(num_iterations):
    optimizer.zero_grad()  # Clear gradients

    output = model(mock_src)
    loss = criterion(output.view(-1, ntoken), mock_target)

    loss.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), 0.5)
    optimizer.step()

    print(f"Iteration {i + 1}/{num_iterations}, Loss: {loss.item():.4f}")

print("\nTraining complete with mock data.")
torch.cuda.memory._dump_snapshot("snapshots/transformer.pickle")
torch.cuda.memory._record_memory_history(enabled=None)

# Print peak VRAM usage
if device.type == "cuda":
    peak_vram_bytes = torch.cuda.max_memory_allocated()
    peak_vram_gb = peak_vram_bytes / (1024**3)
    print(f"\nPeak VRAM usage: {peak_vram_gb:.2f} GB")

# Optional: Run a mock inference step
model.eval()  # Set the model to evaluation mode
with torch.no_grad():
    mock_inference_src = torch.randint(0, ntoken, (seq_len, 1)).to(device)
    inference_output = model(mock_inference_src)
    print(f"\nMock inference output shape: {inference_output.shape}")
    predicted_tokens = torch.argmax(inference_output, dim=-1)
    print(f"Mock predicted tokens (first 5):\n{predicted_tokens[:5].squeeze()}")
