#!/bin/bash
set -e

# Hedef Klasör (Projenin bir üst dizininde oluşturulur)
DEST_DIR="../tapi_clean_export"

echo "🚀 Temiz proje dışa aktarımı başlatılıyor..."
echo "📂 Hedef: $DEST_DIR"

# Varsa eski yedeği temizle
if [ -d "$DEST_DIR" ]; then
    echo "🗑️  Eski klasör temizleniyor..."
    rm -rf "$DEST_DIR"
fi
mkdir -p "$DEST_DIR"

echo "📦 Dosyalar kopyalanıyor..."
echo "   (Python, node_modules, target ve build dosyaları HARIÇ tutuluyor)"

# Rsync ile akıllı kopyalama
# --include kuralları önceliklidir, fakat exclude kuralları spesifik dosyaları engellemek için başa yazılmalıdır.
rsync -av \
    --exclude='node_modules' \
    --exclude='src-tauri/target' \
    --exclude='src-tauri/gen/android/app/build' \
    --exclude='src-tauri/gen/android/.gradle' \
    --exclude='src-tauri/gen/android/build' \
    --exclude='*.py' \
    --exclude='*.env' \
    --exclude='requirements.txt' \
    --exclude='.git' \
    --exclude='.vscode' \
    --exclude='.idea' \
    --exclude='dist' \
    --exclude='build' \
    --exclude='*.log' \
    --include='src/***' \
    --include='src-tauri/***' \
    --include='static/***' \
    --include='scripts/***' \
    --include='lang/***' \
    --include='package.json' \
    --include='yarn.lock' \
    --include='tsconfig.json' \
    --include='vite.config.ts' \
    --include='svelte.config.js' \
    --include='tailwind.config.ts' \
    --include='postcss.config.js' \
    --include='README.md' \
    --include='LICENSE' \
    --exclude='*' \
    --exclude='*.sh' \
    ./ "$DEST_DIR/"

# Kendi scriptini oraya kopyalamasın diye exclude *.sh dedik ama build scriptleri lazım olabilir.
# scripts klasörünü zaten include ile aldık, o yüzden oradakiler gelir.

echo "✅ Dışa aktarım başarıyla tamamlandı!"
echo "📍 Konum: $(realpath $DEST_DIR)"
echo "💾 Toplam Boyut:"
du -sh "$DEST_DIR"
