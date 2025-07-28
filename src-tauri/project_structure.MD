# Project Structure

```
medical-note-generator/
├── icon_gen.bat
├── package.json
├── README.md
├── project_structure.MD
├── binaries/
│   ├── llamafile.exe
│   ├── whisperfile.exe
│   ├── README.txt
│   └── models/
│       ├── med_llama.gguf
│       ├── whisper-tiny.en.gguf
│       └── README.txt
├── src/
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── src-tauri/
│   ├── build.rs
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── main.log
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   ├── default.json
│   │   └── gen/
│   │       └── schemas/
│   │           ├── acl-manifests.json
│   │           ├── capabilities.json
│   │           ├── desktop-schema.json
│   │           └── windows-schema.json
│   ├── icons/
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── 32x32.png
│   │   ├── android-icon-*.png
│   │   ├── apple-icon-*.png
│   │   ├── favicon-*.png
│   │   ├── favicon.ico
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   ├── manifest.json
│   │   ├── ms-icon-*.png
│   │   ├── browserconfig.xml
│   │   └── README.txt
│   └── src/
│       ├── commands.rs
│       └── main.rs
└── target/
    └── debug/
        ├── build/
        ├── deps/
        ├── examples/
        └── incremental/
```