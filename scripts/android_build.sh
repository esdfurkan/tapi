#!/bin/bash

echo "Checking Java environment for Android build..."

CURRENT_JAVA_VER=$(java -version 2>&1 | head -n 1 | awk -F '"' '{print $2}' | cut -d'.' -f1)
echo "Current Java major version: $CURRENT_JAVA_VER"

NEED_SEARCH=false

# Check if version is too new
if [ "$CURRENT_JAVA_VER" -ge "23" ]; then
    echo "Java version is too new for Android Gradle Plugin (detected $CURRENT_JAVA_VER)."
    NEED_SEARCH=true
else
    # Check if we have a compiler (JDK)
    if ! javac --version >/dev/null 2>&1; then
        echo "Current Java installation seems to be lacking 'javac' (JRE only?)."
        NEED_SEARCH=true
    else
         echo "Java version compatible and compiler found."
         # Ensure JAVA_HOME is set to the current java if not set
         if [ -z "$JAVA_HOME" ]; then
             # Try to resolve java location
             JAVA_BIN=$(readlink -f $(which java))
             export JAVA_HOME=$(dirname $(dirname "$JAVA_BIN"))
             echo "guessed JAVA_HOME=$JAVA_HOME"
         fi
    fi
fi

if [ "$NEED_SEARCH" = true ]; then
    echo "Attempting to find a compatible JDK (17-22)..."
    
    # Common paths for Debian/Ubuntu/Fedora
    POTENTIAL_JDKS=(
        "/usr/lib/jvm/java-21-openjdk-amd64"
        "/usr/lib/jvm/java-21-openjdk"
        "/usr/lib/jvm/java-17-openjdk-amd64"
        "/usr/lib/jvm/java-17-openjdk"
        "/usr/lib/jvm/default-java"
    )

    FOUND_JAVA=""
    for path in "${POTENTIAL_JDKS[@]}"; do
        if [ -d "$path" ] && [ -x "$path/bin/javac" ]; then
            # Verify javac is actually working (not a broken symlink)
            if "$path/bin/javac" --version >/dev/null 2>&1; then
                 # Verify version
                 VER=$("$path/bin/java" -version 2>&1 | head -n 1 | awk -F '"' '{print $2}' | cut -d'.' -f1)
                 if [ "$VER" -le "22" ] && [ "$VER" -ge "17" ]; then
                     FOUND_JAVA="$path"
                     break
                 fi
            else
                 echo "Skipping broken JDK at $path (javac found but not working)"
            fi
        fi
    done

    if [ -n "$FOUND_JAVA" ]; then
        export JAVA_HOME="$FOUND_JAVA"
        export PATH="$JAVA_HOME/bin:$PATH"
        echo "Switched to compatible JDK: $FOUND_JAVA"
        
        # Verify javac exists
        if [ -x "$JAVA_HOME/bin/javac" ]; then
             echo "javac compiler found."
        else
             echo "WARNING: javac not found in $JAVA_HOME/bin. This might be a JRE."
        fi
    else
        echo "ERROR: Could not find a compatible JDK (17-22). Please install a full JDK (e.g. openjdk-21-jdk)."
        echo "Current candidates checked: ${POTENTIAL_JDKS[*]}"
        exit 1
    fi
else
    echo "Java version looks compatible."
fi

# Force Gradle to detect the java toolchain from JAVA_HOME logic
# We use GRADLE_OPTS to pass system properties to Gradle
export GRADLE_OPTS="-Dorg.gradle.daemon=false -Dorg.gradle.java.installations.auto-detect=false -Dorg.gradle.java.installations.paths=$JAVA_HOME"

echo "Starting build with JAVA_HOME=$JAVA_HOME"
echo "GRADLE_OPTS=$GRADLE_OPTS"

# --- KEYSTORE CHECK START ---
# Check if keystore exists, if not create it automatically
KEYSTORE_FILE="$HOME/upload-keystore.jks"
KEY_PROPERTIES="src-tauri/gen/android/key.properties"

if [ ! -f "$KEYSTORE_FILE" ] || [ ! -f "$KEY_PROPERTIES" ]; then
    echo "⚠️  Keystore not found. Creating one automatically..."
    if [ -f "scripts/create_keystore.sh" ]; then
        bash scripts/create_keystore.sh
        if [ $? -ne 0 ]; then
            echo "❌ Failed to create keystore. Falling back to debug build."
            FORCE_DEBUG=true
        fi
    else
        echo "❌ create_keystore.sh not found. Falling back to debug build."
        FORCE_DEBUG=true
    fi
fi
# --- KEYSTORE CHECK END ---

# --- NDK FIX START ---
# 1. Check if NDK_HOME is invalid and unset it
if [ -n "$NDK_HOME" ] && [ ! -d "$NDK_HOME" ]; then
    echo "⚠️  WARNING: NDK_HOME points to non-existent directory '$NDK_HOME'. Unsetting it."
    unset NDK_HOME
fi

# 2. Setup SDKMANAGER
if [ -z "$ANDROID_HOME" ]; then
    export ANDROID_HOME="$HOME/Android/Sdk"
fi

SDKMANAGER=""
if [ -f "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
elif [ -f "$ANDROID_HOME/cmdline-tools/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/cmdline-tools/bin/sdkmanager"
elif [ -f "$ANDROID_HOME/tools/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/tools/bin/sdkmanager"
fi

# 3. Try to Auto-Install NDK if missing or try to locate it
if [ -z "$NDK_HOME" ]; then
    # Look for existing NDKs
    if [ -d "$ANDROID_HOME/ndk" ]; then
        LATEST_NDK=$(ls -d "$ANDROID_HOME/ndk/"* 2>/dev/null | sort -V | tail -n 1)
        if [ -n "$LATEST_NDK" ] && [ -d "$LATEST_NDK" ]; then
            export NDK_HOME="$LATEST_NDK"
            echo "✅ Auto-assigned NDK_HOME=$NDK_HOME"
        fi
    fi
fi

if [ -z "$NDK_HOME" ] && [ -n "$SDKMANAGER" ]; then
    echo "⚠️  NDK not found. Attempting to install via sdkmanager..."
    # Install specific LTS version or latest
    yes | "$SDKMANAGER" --install "ndk;26.1.10909125" --sdk_root="$ANDROID_HOME" >/dev/null 2>&1 || true
    
    # Check again
    LATEST_NDK=$(ls -d "$ANDROID_HOME/ndk/"* 2>/dev/null | sort -V | tail -n 1)
    if [ -n "$LATEST_NDK" ]; then
        export NDK_HOME="$LATEST_NDK"
        echo "✅ Installed and assigned NDK_HOME=$NDK_HOME"
    else
        echo "❌ Failed to install NDK. Android build might fail."
    fi
fi
# --- NDK FIX END ---

# Auto-accept Android Licenses
if [ -z "$ANDROID_HOME" ]; then
    export ANDROID_HOME="$HOME/Android/Sdk"
fi

SDKMANAGER=""
if [ -f "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
elif [ -f "$ANDROID_HOME/cmdline-tools/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/cmdline-tools/bin/sdkmanager"
elif [ -f "$ANDROID_HOME/tools/bin/sdkmanager" ]; then
    SDKMANAGER="$ANDROID_HOME/tools/bin/sdkmanager"
fi

if [ -n "$SDKMANAGER" ]; then
    echo "Creating license file auto-accept..."
    # We use 'yes' to accept all licenses
    # Using --sdk_root to prevent "Could not determine SDK root" errors
    yes | "$SDKMANAGER" --licenses --sdk_root="$ANDROID_HOME" >/dev/null 2>&1 || true
    echo "Licenses check completed."
else
    echo "WARNING: sdkmanager not found at $ANDROID_HOME. Skipping license auto-accept."
fi

# Increase Node memory limit to prevent OOM (Exit code 137) during frontend build
# 4GB might be too aggressive if system RAM is low. Trying 2GB or whatever is safe.
# We also reduce Vite concurrency.
export NODE_OPTIONS="--max-old-space-size=4096"
export UV_THREADPOOL_SIZE=4

echo "Set NODE_OPTIONS=$NODE_OPTIONS to prevent OOM"

# Check if frontend is prebuilt
EXTRA_ARGS=""
if [ "$PREBUILT_FRONTEND" = "true" ]; then
    echo "Skipping frontend build (PREBUILT_FRONTEND=true)"
    # IMPORTANT: JSON inside bash string needs careful escaping for Tauri CLI
    EXTRA_ARGS="--config {\"build\":{\"beforeBuildCommand\":\"\"}}"
fi

# Run the build (DEBUG MODE - no signing required)
# Use --debug flag to skip release signing
# Only build arm64 and armv7 (skip x86/x86_64 emulator targets to reduce size)
yarn tauri android build $EXTRA_ARGS --debug --target aarch64 --target armv7
