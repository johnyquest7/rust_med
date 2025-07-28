@echo off
echo Creating real placeholder icon files...

cd src-tauri

:: Create icons directory if it doesn't exist
if not exist "icons" mkdir icons
cd icons

:: Remove any existing placeholder files
del /q *.png *.ico *.icns 2>nul

echo Creating simple colored PNG files as placeholder icons...

:: Create a simple PowerShell script to generate basic PNG files
powershell -Command ^
"Add-Type -AssemblyName System.Drawing; ^
$bmp32 = New-Object System.Drawing.Bitmap(32, 32); ^
$g32 = [System.Drawing.Graphics]::FromImage($bmp32); ^
$g32.Clear([System.Drawing.Color]::Blue); ^
$g32.DrawString('M', (New-Object System.Drawing.Font('Arial', 18)), [System.Drawing.Brushes]::White, 8, 5); ^
$bmp32.Save('32x32.png', [System.Drawing.Imaging.ImageFormat]::Png); ^
$g32.Dispose(); ^
$bmp32.Dispose(); ^
$bmp128 = New-Object System.Drawing.Bitmap(128, 128); ^
$g128 = [System.Drawing.Graphics]::FromImage($bmp128); ^
$g128.Clear([System.Drawing.Color]::Blue); ^
$g128.DrawString('MED', (New-Object System.Drawing.Font('Arial', 36)), [System.Drawing.Brushes]::White, 25, 45); ^
$bmp128.Save('128x128.png', [System.Drawing.Imaging.ImageFormat]::Png); ^
$bmp128.Save('128x128@2x.png', [System.Drawing.Imaging.ImageFormat]::Png); ^
$g128.Dispose(); ^
$bmp128.Dispose(); ^
Write-Host 'PNG files created successfully';"

:: Copy 32x32.png as ico file (basic approach)
copy 32x32.png icon.ico >nul 2>&1

:: Create a dummy icns file for macOS (just copy the PNG)
copy 128x128.png icon.icns >nul 2>&1

echo.
echo Icon files created in src-tauri/icons/:
dir /b *.png *.ico *.icns

echo.
echo Note: These are basic placeholder icons. For production, use proper ICO and ICNS formats.
echo You can use online converters to create proper icon files from PNG images.

cd ..\..
pause