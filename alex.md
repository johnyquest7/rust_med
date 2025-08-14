```

# Assume:
# (a) node is set up

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# restart terminal
rustup update
cargo install tauri-cli

mkdir -p binaries
curl -L -o llamafile "https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3"
wget https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile
mv llamafile binaries
mv whisper-tiny.en.llamafile whisperfile
chmod +x binaries/llamafile
chmod +x binaries/whisperfile
wget -O whisper-tiny.en.gguf https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
mv whisper-tiny.en.gguf binaries/models

# Download med_gguf and put in binaries/models/med_llama.gguf
# Link to models: https://huggingface.co/garcianacho/MedLlama-2-7B-GGUF/tree/main

# convert icon_gen.bat to mac version
.icon_gen.sh

npm run dev
```
