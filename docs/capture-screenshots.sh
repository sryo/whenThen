#!/bin/bash
# Capture screenshots with demo data for docs

set -e

APP_SUPPORT=~/Library/Application\ Support/app.when
DEMO_DATA="$(dirname "$0")/demo-data"
SCREENSHOTS="$(dirname "$0")/screenshots"
APP="/Applications/When.app"

# Create screenshots dir
mkdir -p "$SCREENSHOTS"

# Backup current data
echo "Backing up current data..."
for f in sources.json interests.json playlets.json settings.json; do
  if [ -f "$APP_SUPPORT/$f" ]; then
    cp "$APP_SUPPORT/$f" "$APP_SUPPORT/$f.bak"
  fi
done

# Copy demo data
echo "Installing demo data..."
cp "$DEMO_DATA"/*.json "$APP_SUPPORT/"

# Create demo mode marker (triggers demo inbox data on startup)
touch "$APP_SUPPORT/demo_mode"

# Kill existing instance
pkill -f When 2>/dev/null || true
sleep 1

# Launch app
echo "Launching app..."
"$APP/Contents/MacOS/when" &
sleep 3

echo ""
echo "=== SCREENSHOT CAPTURE ==="
echo "Click tray icon to open window first!"
echo ""
echo "For each screenshot: navigate to the view, then CLICK THE WINDOW when crosshair appears."
echo ""

echo ">>> 1. Go to INBOX (Cmd+1), then click window..."
sleep 5
screencapture -ow "$SCREENSHOTS/inbox.png"
echo "Captured inbox.png"

echo ">>> 2. Go to PLAYLETS (Cmd+2), then click window..."
sleep 5
screencapture -ow "$SCREENSHOTS/playlets.png"
echo "Captured playlets.png"

echo ">>> 3. Open EDIT PLAYLET modal, then click window..."
sleep 5
screencapture -ow "$SCREENSHOTS/edit-playlet.png"
echo "Captured edit-playlet.png"

echo ""
echo "Done!"

# Remove demo mode marker
rm -f "$APP_SUPPORT/demo_mode"

# Restore original data
echo "Restoring original data..."
for f in sources.json interests.json playlets.json settings.json; do
  if [ -f "$APP_SUPPORT/$f.bak" ]; then
    mv "$APP_SUPPORT/$f.bak" "$APP_SUPPORT/$f"
  else
    rm -f "$APP_SUPPORT/$f"
  fi
done

echo "Done! Screenshots saved to: $SCREENSHOTS"
