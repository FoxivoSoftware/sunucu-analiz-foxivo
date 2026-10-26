#!/usr/bin/env bash
# Writes the release notes: a download table on top, then the Windows warning,
# verification steps and (if present) VirusTotal links from $VT_FILE.
# usage: release-notes.sh <tag> <owner/repo> [vt-links.md]
set -euo pipefail

tag="$1"
repo="$2"
vt_file="${3:-}"
base="https://github.com/$repo/releases/download/$tag"

# tr: gh on Windows ends lines with \r, which would break the suffix matches below
assets=$(gh release view "$tag" --repo "$repo" --json assets --jq '.assets[] | "\(.name)\t\(.size)"' | tr -d '\r')

# first asset matching a suffix -> "name<TAB>size"
pick() { printf '%s\n' "$assets" | awk -F'\t' -v re="$1" '$1 ~ re && $1 !~ /\.sig$/ { print; exit }'; }
link() {
  local row name size
  row=$(pick "$1")
  [ -z "$row" ] && { echo "—"; return; }
  name=${row%%$'\t'*}
  size=${row##*$'\t'}
  printf '[**%s indir**](%s/%s) · %s MB' "$2" "$base" "$name" "$(awk "BEGIN{printf \"%.1f\", $size/1048576}")"
}

cat <<EOF
## İndir

| Sistem | İndirme |
| --- | --- |
| **Windows** 10 / 11 | $(link '_x64-setup[.]exe$' 'Kurulum dosyasını') |
| **macOS** (Intel ve Apple Silicon) | $(link '[.]dmg$' 'DMG dosyasını') |
| **Linux** | $(link '[.]AppImage$' 'AppImage') <br> $(link '[.]deb$' '.deb paketini') |

> **Windows "bilinmeyen yayımcı" uyarısı gösterirse:** **Ek bilgi**'ye, ardından **Yine de çalıştır**'a tıkla. Bu uyarı virüs bulunduğu anlamına gelmez; uygulama ücretli kod imzası kullanmadığı için çıkar. Neden çıktığı ve adım adım anlatım: [Windows kurulum rehberi](https://github.com/$repo#smartscreen)

Zaten kuruluysa yeni sürümü uygulama kendisi bulur: **Ayarlar › Hakkında › Güncellemeleri denetle**.

## Doğrulama

- Dosya özetleri: [SHA256SUMS]($base/SHA256SUMS) — \`sha256sum -c SHA256SUMS --ignore-missing\`
EOF

if [ -n "$vt_file" ] && [ -s "$vt_file" ]; then
  printf -- '- VirusTotal tarama sonuçları:\n\n'
  cat "$vt_file"
fi

cat <<EOF

<sub>Listedeki \`.sig\` ve \`latest.json\` dosyaları uygulama içi güncelleme içindir, indirmen gerekmez.</sub>
EOF
