# given PyTorch dump at ./snapshots/large/transformer.pickle
python parse_dump.py -p snapshots/large/transformer.pickle -o ./dumpjson -d 0 -z
# this outputs to ./snap.zip

cd snap-rs

# then load the zipped dump
cargo run -r --bin repl -- --zip ../dumpjson/snap.zip