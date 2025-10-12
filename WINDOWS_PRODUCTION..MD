# Production Build Guide for Windows

This guide details the steps to create a production-ready Windows distribution of the Medical Note Generator application.

## Prerequisites

Before building for production, ensure you have:

- **Node.js** (v18 or later)
- **Rust** and **Cargo** (latest stable version)
- **Windows SDK** (for creating MSI installers)
- All project dependencies installed

## Pre-Build Checklist

### 1. Verify Icon Files

Ensure all icon files in `src-tauri/icons/` are valid and in the correct format:
- `icon.ico` - Windows icon (must be in ICO 3.00 format)
- `icon.png` - Base icon for other platforms
- Other size variants (32x32, 128x128, etc.)

If you encounter icon errors, regenerate icons using:
```bash
npx @tauri-apps/cli icon path/to/source-icon.png
```

### 2. Update Version Information

Update version numbers in these files:
- `src-tauri/tauri.conf.json` - Update `version` field
- `package.json` - Update `version` field
- `src-tauri/Cargo.toml` - Update `version` field

### 3. Configure Application Metadata

Edit `src-tauri/tauri.conf.json` to set:
```json
{
  "productName": "Medical Note Generator",
  "version": "0.1.0",
  "identifier": "com.medical.notegenerator",
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

## Building for Production

### Step 1: Install Dependencies

From the project root directory:

```bash
# Install root dependencies
npm install

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### Step 2: Build the Application

Run the production build command:

```bash
npm run build
```

Or directly with Tauri:

```bash
cargo tauri build
```

### Build Process Details

The build process performs the following steps:

1. **Frontend Build**
   - Runs `vite build` in the `frontend/` directory
   - Creates optimized, minified production bundle
   - Outputs to `frontend/build/`
   - Uses `@sveltejs/adapter-static` for static site generation

2. **Rust Compilation**
   - Compiles Rust code in release mode with optimizations
   - Includes all dependencies (Tauri, rusqlite, argon2, aes-gcm, etc.)
   - Embeds frontend bundle into the executable

3. **Installer Creation**
   - Creates Windows MSI installer
   - Creates NSIS installer (.exe)
   - Bundles application resources and icons

### Build Output Locations

After a successful build, installers are located at:

```
src-tauri/target/release/bundle/
├── msi/
│   └── Medical Note Generator_0.1.0_x64_en-US.msi
└── nsis/
    └── Medical Note Generator_0.1.0_x64-setup.exe
```

The standalone executable (without installer) is at:
```
src-tauri/target/release/OnDevice_Medical_Note.exe
```

## Distribution Options

### Option 1: MSI Installer (Recommended for Enterprise)

- **File**: `Medical Note Generator_0.1.0_x64_en-US.msi`
- **Pros**: Standard Windows installer, IT-friendly, supports Group Policy
- **Use Case**: Enterprise deployment, corporate environments

### Option 2: NSIS Installer (Recommended for End Users)

- **File**: `Medical Note Generator_0.1.0_x64-setup.exe`
- **Pros**: Smaller file size, customizable UI, better UX
- **Use Case**: General distribution, consumer applications

### Option 3: Portable Executable

- **File**: `OnDevice_Medical_Note.exe`
- **Pros**: No installation required, can run from USB
- **Cons**: Requires manual setup, no auto-updates
- **Use Case**: Portable/testing scenarios

## Post-Build Steps

### 1. Test the Installer

Before distribution, test the installer on a clean Windows machine:

1. Run the installer
2. Verify the application launches correctly
3. Test core functionality:
   - Audio recording
   - Transcription (Whisper)
   - Note generation (LLaMA)
   - Database operations
   - Encryption/authentication
4. Check the setup wizard downloads AI models correctly
5. Verify the application can be uninstalled cleanly

### 2. Code Signing (Optional but Recommended)

For production distribution, sign your executables to avoid Windows SmartScreen warnings:

```bash
# Using signtool (requires valid code signing certificate)
signtool sign /f "certificate.pfx" /p "password" /t "http://timestamp.digicert.com" "path/to/installer.exe"
```

Update `tauri.conf.json` with certificate information:
```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### 3. Create Distribution Package

Create a release package with:

```
medical-note-generator-v0.1.0-windows/
├── Medical Note Generator_0.1.0_x64_en-US.msi
├── Medical Note Generator_0.1.0_x64-setup.exe
├── README.md (user documentation)
├── LICENSE.txt
└── CHANGELOG.md
```

## Important Notes

### AI Models

The application does NOT include AI models in the installer. Models are downloaded automatically on first launch:

- **Whisper Model** (~83 MB) - Speech-to-text transcription
- **Llamafile Runtime** (~293 MB) - LLM runtime
- **Whisper Tiny Model** (~141 MB) - Whisper model file
- **MedLlama Model** (~3.7 GB) - Medical note generation

Total download size: ~4.2 GB on first launch

Models are stored in:
```
C:\Users\{USERNAME}\AppData\Local\com.medical.notegenerator\binaries\
```

### Database Location

User data is stored in:
```
C:\Users\{USERNAME}\AppData\Local\com.medical.notegenerator\medical_notes.db
```

All patient notes are encrypted using AES-256-GCM with a user-provided password.

### Windows Compatibility

The application is compatible with:
- Windows 10 (64-bit) version 1809 or later
- Windows 11 (64-bit)

### File Paths and Windows Compatibility

The application automatically handles Windows path conversion for llamafile/whisperfile compatibility:
- Windows paths (`C:\Users\...`) are converted to Unix-style (`C:/Users/...`)
- Executable files automatically get `.exe` extension on Windows
- All paths work correctly with llamafile's internal path handling

## Troubleshooting Build Issues

### Icon Format Error

```
error RC2175 : resource file is not in 3.00 format
```

**Solution**: Regenerate icons using Tauri's icon command or use a tool like ImageMagick to create proper ICO format.

### Missing Dependencies

```
error: linker `link.exe` not found
```

**Solution**: Install Visual Studio Build Tools with C++ development tools.

### Frontend Build Fails

**Solution**:
```bash
cd frontend
npm install
npm run build
cd ..
```

### Cargo Build Fails

**Solution**:
```bash
cd src-tauri
cargo clean
cargo build --release
```

## Development vs Production

| Aspect | Development | Production |
|--------|-------------|------------|
| Build Command | `npm run dev` | `npm run build` |
| Optimization | None (fast compile) | Full optimization |
| File Size | Larger | Smaller |
| Performance | Slower | Faster |
| Debug Info | Included | Stripped |
| Hot Reload | Yes | No |
| Bundle Location | Not created | `target/release/bundle/` |

## Release Checklist

Before releasing a new version:

- [ ] Update version numbers in all config files
- [ ] Test all functionality in development mode
- [ ] Run production build
- [ ] Test installer on clean Windows machine
- [ ] Verify AI model downloads work
- [ ] Test authentication and encryption
- [ ] Test audio recording and transcription
- [ ] Test medical note generation
- [ ] Check database operations
- [ ] Verify uninstall process
- [ ] Sign executables (if applicable)
- [ ] Create release notes
- [ ] Tag git commit with version number

## Support and Documentation

For end-user documentation, refer to:
- `README.md` - General overview and features
- `CLAUDE.md` - Development documentation
- User manual (to be created for production release)

## Security Considerations

When distributing:
1. Never include test credentials or API keys
2. Ensure all dependencies are up-to-date
3. Run security audits: `npm audit` and `cargo audit`
4. Consider code signing for trust and authenticity
5. Document encryption and privacy features for users
6. Provide clear HIPAA compliance information (if applicable)

## Performance Optimization Tips

For optimal performance in production:
1. Ensure AI models are downloaded to fast storage (SSD)
2. Recommend minimum 16GB RAM for smooth LLM operation
3. Advise users to close other applications during note generation
4. Monitor CPU usage during transcription/generation
5. Consider providing performance settings in future versions

## Future Improvements

Consider for future releases:
- Auto-update functionality (Tauri updater)
- Smaller model options for faster processing
- Cloud backup for encrypted notes
- Multi-language support
- macOS and Linux builds
- Digital signatures on installers
- Telemetry for crash reporting (with user consent)
