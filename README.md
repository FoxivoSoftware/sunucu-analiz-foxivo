# Sunucu Analiz - Foxivo

Minecraft sunucu klasörünü seç, saniyeler içinde sağlık raporunu gör. Hazır paketlerdeki örnek değerleri, bozuk Türkçe karakterleri ve şüpheli eklentileri bulur; sen onaylamadan hiçbir dosyaya dokunmaz.

Ücretsiz ve açık kaynak. Windows, macOS ve Linux.

> **English:** Pick your Minecraft server folder and get a health report in seconds. Read-only by default, nothing changes without your approval. Free and open source.

## Neler yapıyor

- **Güvenlik taraması:** eklenti jar'larını çalıştırmadan inceler. Sınıf dosyalarındaki gerçek metot çağrılarına bakar, şüpheli kalıpları nedenleriyle gösterir.
- **Resmi sürüm doğrulaması:** her jar'ın SHA-512 özetini Modrinth'teki resmi yüklemelerle karşılaştırır. Birebir aynıysa dosyaya dokunulmamıştır. Yalnızca dosya özeti gönderilir, ayarlardan kapatılabilir.
- **Eklenti güncellemeleri:** doğrulanan eklentiler için sunucu yazılımına ve Minecraft sürümüne uygun yeni kararlı sürümleri gösterir.
- **Örnek değerleri değiştir:** `sunucuadiniz.com` gibi yer tutucuları ve paketin kendi adreslerini (`www.`, `magaza.`, `discord.` ...) tüm config ve dil dosyalarında tek formdan değiştirir.
- **Türkçe karakter onarımı:** `Ã¼`, `ÅŸ` gibi bozulmaları ve Windows-1254 ile kaydedilmiş dosyaları bulur, güvenle düzeltilebilenleri önerir.
- **Optimizasyon önerileri:** hiçbiri kendiliğinden uygulanmaz.
- **Anticheat yorumu:** çakışmalar ve öneriler.
- **Yedek ve geri alma:** her değişiklikten önce otomatik yedek, tek tıkla geri yükleme.

## Güvenlik ilkeleri

1. Varsayılan mod salt okunur.
2. Her değişiklik önce plan olarak gösterilir, onaylanmadan uygulanmaz.
3. Yazmadan önce yedek, atomik yazma, sözdizimi doğrulama; hata olursa otomatik geri alma.
4. Sunucu çalışıyorsa yazma işlemleri durur.
5. `world*`, `playerdata`, jar ve veritabanı dosyalarına asla yazılmaz.
6. Dosya içerikleri hiçbir zaman ağa gönderilmez.

Güvenlik taraması bilinen şüpheli kalıpları arar; "şüpheli bulgu yok" demek "kesinlikle güvenli" demek değildir. Kesin cevap için resmi sürüm doğrulamasına bakın.

## İndirme ve güncelleme

Sürümler [Releases](../../releases) sayfasında, her sürümde `SHA256SUMS` dosyası ve VirusTotal tarama bağlantılarıyla birlikte.

Uygulama içinden güncellemek için **Ayarlar › Hakkında › Güncellemeleri denetle**. Güncelleme paketleri imzalıdır; imzası tutmayan bir dosya kurulmaz.

Uygulama ücretli bir kod imzası kullanmıyor, bu yüzden ilk açılışta uyarı görebilirsin:

- **Windows:** "Windows bilgisayarınızı korudu" → **Ek bilgi** → **Yine de çalıştır**
- **macOS:** uygulamaya sağ tıkla → **Aç** → **Aç**

İndirdiğin dosyayı doğrulamak için:

```bash
sha256sum -c SHA256SUMS --ignore-missing
```

Pencereyi kapatınca uygulama sistem tepsisinde çalışmaya devam eder; tamamen kapatmak için tepsi simgesinde **Çıkış**. Bu davranış ayarlardan değiştirilebilir.

## Geliştirme

Gerekenler: Node.js 22+, Rust (stable), [Tauri önkoşulları](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

Arayüzü tarayıcıda denemek için `npm run dev` yeterli; tarama bu modda örnek bir raporla çalışır.

### Yapı

```
crates/core/    tarayıcılar ve güvenli yazma hattı (yedek → atomik yazma → doğrulama → geri alma)
crates/cli/     komut satırı aracı
src/            Svelte arayüzü
src-tauri/      Tauri kabuğu, tepsi simgesi, kurulum ayarları
locales/        tr.json, en.json
```

Metinler `locales/` altında; iki dil dosyasının anahtarları `npm run check` ile karşılaştırılır.

### Komut satırı

Hosting firmaları ve CI için aynı tarayıcı komut satırından da çalışır:

```bash
cargo run -p analiz-cli -- scan /sunucu/klasoru --json --online --fail-on high
```

`--fail-on` verilen seviyede bulgu varsa 3 koduyla çıkar. `--online` resmi sürüm doğrulamasını açar.

### Testler

```bash
cargo test --workspace
```

### Sürüm yayınlama

`v` ile başlayan bir etiket gönderildiğinde GitHub Actions üç platform için kurulum dosyalarını üretir, güncelleme paketlerini imzalar ve taslak bir sürüm oluşturur. Depo ayarlarında şu gizli değerler tanımlı olmalı:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (anahtar şifresizse boş)
- `VT_API_KEY` (isteğe bağlı) — tanımlıysa kurulum dosyaları VirusTotal'e gönderilir ve sonuç bağlantıları sürüm notlarına eklenir

## Lisans

MIT. Geliştiren: [Foxivo](https://foxivo.com.tr)
