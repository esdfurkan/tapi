#!/bin/bash
set -e

echo "🚀 Başlatılıyor: Tüm Platformlar İçin Build..."

# Flag check
ONLY_X86=false
ONLY_ARM=false
ONLY_WIN=false
ONLY_ANDROID=false

for arg in "$@"; do
  case $arg in
    --only-x86) ONLY_X86=true ;;
    --only-arm) ONLY_ARM=true ;;
    --only-win) ONLY_WIN=true ;;
    --only-android) ONLY_ANDROID=true ;;
  esac
done

# If no flags, build all
if [ "$ONLY_X86" = false ] && [ "$ONLY_ARM" = false ] && [ "$ONLY_WIN" = false ] && [ "$ONLY_ANDROID" = false ]; then
  ALL=true
else
  ALL=false
fi

# --- 0. Environment Checks ---
if [ "$ONLY_WIN" = true ] || [ "$ALL" = true ]; then
    echo "🔍 Windows (MinGW) gereksinimleri kontrol ediliyor..."
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "❌ HATA: MinGW derleyicisi bulunamadı."
    fi
    if ! rustup target list | grep "x86_64-pc-windows-gnu (installed)" &> /dev/null; then
        rustup target add x86_64-pc-windows-gnu
    fi
fi

if [ "$ONLY_ARM" = true ] || [ "$ALL" = true ]; then
    if ! rustup target list | grep "aarch64-unknown-linux-gnu (installed)" &> /dev/null; then
        rustup target add aarch64-unknown-linux-gnu
    fi
fi

# --- 1. Frontend Build ---
echo "📦 Building Frontend..."
yarn install
yarn build
export PREBUILT_FRONTEND=true

# --- 2. Linux Build (x86_64) ---
if [ "$ONLY_X86" = true ] || [ "$ALL" = true ]; then
    echo "🐧 Building for Linux (x86_64)..."
    yarn tauri build --bundles deb,appimage,rpm
fi

# --- 2.1 Linux Build (ARM64) ---
if [ "$ONLY_ARM" = true ] || [ "$ALL" = true ]; then
    if command -v aarch64-linux-gnu-gcc &> /dev/null; then
        echo "🐧 Building for Linux (ARM64)..."
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu/pkgconfig
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        yarn tauri build --target aarch64-unknown-linux-gnu --bundles deb,appimage
    else
        echo "⚠️  Skipping ARM64 build: Cross-compiler (aarch64-linux-gnu-gcc) not found."
    fi
fi

# --- 3. Windows Build ---
if [ "$ONLY_WIN" = true ] || [ "$ALL" = true ]; then
    echo "🪟 Building for Windows (.exe)..."
    yarn tauri build --target x86_64-pc-windows-gnu
fi

# --- 4. Android Build ---
if [ "$ONLY_ANDROID" = true ] || [ "$ALL" = true ]; then
    echo "🤖 Building for Android..."
    if [ -f "scripts/android_build.sh" ]; then
        bash scripts/android_build.sh
    else
        yarn tauri android build
    fi
fi

echo "✅ BUILD TAMAMLANDI!"

