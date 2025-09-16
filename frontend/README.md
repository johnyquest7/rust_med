# Medical Note Generator

This project allows medical professionals to:

- Turn on recording during patient visits
- Record the visit
- Get a transcription of the visit with whisper
- Generate a SOAP medical note from the transcription
- Medical professionals can then navigate their notes.

Eventually this project will be migrated to use Tauri and wrapped as an app, but for now, it's a web only project.

## Developing

```sh
# Install dependencies
npm install

# Run the dev server
npm run dev

# Or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

```sh
npm run build
```
