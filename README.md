A simple tauri based app made with svelte and rust.

## Installation

Go to release page and download the latest release for your platform.

## Preqrequisites

- Download webview2 if you need
- https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download

## Usage

Open the app and follow the instructions.
First: Select a folder to translate.
Second: Open settings and select a target language.
Third: Enter a valid api key (Get a api key here: https://toriitranslate.com/api).
Fourth: Select a model and font. Click done
Fifth: Click to Start Translation button and wait for it to finish.
Sixth: Open the output folder to find your translated files.

## Photo
img1
img2
img3


## Features

- You can see which files you have translated and which you haven't by hashing the file using the blake3 algorithm. If you want to translate it again, you will need to delete the .f_history file in the folder.

- API key is encrypted with ChaCha20-Poly1305.  This protocol is used in VPNs; once you enter the API key, you cannot see it again. Even if the file storing the API key is copied, it will not work on another computer because an encryption key is created based on the UID specific to your computer. If you format your computer or install another operating system, it will not work on your newly installed OS either. That means even if I wanted to, I couldn't solve it until this algorithm is broken.
https://docs.rs/machine-uid/latest/machine_uid/
https://en.wikipedia.org/wiki/ChaCha20-Poly1305


- Custom model and font (only allowed api models )
- Download nhentai pixiv mangadex and more website (not work all website) 

## Build requirements
apt-get update && \
            apt-get install -y \
              libwebkit2gtk-4.1-dev \
              libgtk-3-dev \
              build-essential \
              curl \
              wget \
              unzip \
              file \
              libssl-dev \
              libayatana-appindicator3-dev \
              librsvg2-dev \
              mingw-w64 \
              nsis \
              jq \
              openjdk-21-jdk && break

https://developer.android.com/studio
https://developer.android.com/ndk/downloads
https://v2.tauri.app/distribute/sign/android/
https://nodejs.org/en/download
https://rustup.rs/


## How to own build

1. Clone the repo
2. Install tauri yarn and rust
3. use `yarn install` to install dependencies
4. Go to src-tauri folder and run `cargo update`
5. Go to main folder and run `yarn build:all`
6. Wait for dependencies to be downloaded and installed
7. Wait for compilation to finish
8. Check logs and where the binary is compiled
9. install and enjoy

## Build requirements

A linux machine 
Good internet connection for downloading dependencies
Good cpu and ram for compiling (Because need fast compile speed) i recommend at least 20GB free storage 8gb ram 4 or more cpu more you have, the faster the compilation time will be inversely proportional.



## Tested platforms

- Windows 11 ✅
- Linux Appimage ✅
- Linux deb ?
- Linux rpm ?
- Android ✅ (There are a few known issues)
- iOS ?
- macOS ?
- Web ?

## Arm64 support

For now, I'll put a compiled version in place of the release, even though it doesn't have support. It may work fine, but I can't test it because I don't have an ARM device. The same goes for Mac or iOS. If you have these devices, you'll need to test it yourself. 

## A brief note

First of all, yes, this code was written with AI, and I'm the only one who reviewed it. I'm not doing this for profit, and I don't think anyone else would write this by hand. If they did, it would have been shared within a year.  
The reason I did this was because the plugin was freezing my weak phone. When it became available to use via API, I saw it as an opportunity and used it. It was a great experience for me. You can thoroughly review all the application's code and decide for yourself whether to install it or not. 
If you're wondering why there are few commits, I'm updating it on my Gitea server until it's stable for testing or, as it's commonly called, beta or canary, and I'll share the final version here or start doing so.