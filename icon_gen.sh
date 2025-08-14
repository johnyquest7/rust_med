#!/bin/bash

echo "Creating real placeholder icon files..."

cd src-tauri

# Create icons directory if it doesn't exist
if [ ! -d "icons" ]; then
    mkdir -p icons
fi
cd icons

# Remove any existing placeholder files
rm -f *.png *.ico *.icns 2>/dev/null

echo "Creating simple colored PNG files as placeholder icons..."

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick not found. Installing via Homebrew..."
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Please install Homebrew first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
    brew install imagemagick
fi

# Create 32x32 icon with 'M' text in RGBA format
convert -size 32x32 xc:blue \
    -alpha set \
    -gravity center \
    -pointsize 18 \
    -fill white \
    -draw "text 0,0 'M'" \
    -define png:format=png32 \
    32x32.png

# Create 128x128 icon with 'MED' text in RGBA format
convert -size 128x128 xc:blue \
    -alpha set \
    -gravity center \
    -pointsize 36 \
    -fill white \
    -draw "text 0,0 'MED'" \
    -define png:format=png32 \
    128x128.png

# Create 128x128@2x icon (same as 128x128 for now)
cp 128x128.png 128x128@2x.png

# Create a basic ico file (copy PNG as placeholder)
cp 32x32.png icon.ico

# Create a basic icns file (copy PNG as placeholder)
cp 128x128.png icon.icns

# Create icon.png from 32x32.png (already in RGBA format)
cp 32x32.png icon.png

echo ""
echo "Icon files created in src-tauri/icons/:"
ls -la *.png *.ico *.icns

echo ""
echo "Note: These are basic placeholder icons. For production, use proper ICO and ICNS formats."
echo "You can use online converters or tools like 'iconutil' to create proper icon files from PNG images."

cd ../..

echo ""
echo "Press Enter to continue..."
read
