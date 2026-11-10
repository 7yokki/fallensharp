# Fallen++

Fallen'ın yeteneklerini koruyup sözdizimini Python'a yaklaştıran, doğrudan x86-64
assembly üreten deneysel bir programlama dili ve derleyicisi. Sürüm: **0.1.0**.

Fallen++ kaynak dosyaları `.fpp` uzantılıdır. Derleyici `.fpp` → GNU assembly →
`as` → `ld` zincirini kullanır; araya C derleyicisi girmez.

## Kurulum

Rust araç zinciri ve `binutils` (`as`, `ld`) gerekir.

```bash
git clone https://github.com/7yokki/fallensharp.git
cd fallensharp
cargo build --release
```

Derleyici `target/release/fallenpp` altında oluşur.

## Kullanım

```bash
target/release/fallenpp build dosya.fpp -o cikti
target/release/fallenpp run dosya.fpp
target/release/fallenpp build dosya.fpp -S
target/release/fallenpp build dosya.fpp -o cikti --stats
target/release/fallenpp check dosya.fpp
target/release/fallenpp version
```

## Sözdizimi

```fpp
import system.exec

fn main():
    stdout("Merhaba\n")

    loop:
        cmd string => stdin("Komut yazınız: ")
        if cmd equal 0:
            break
        end
        output string => system.exec(cmd, output=True)
        stdout(output)
    do
end
```

Kurallar:

- Bloklar `:` ile başlar; gövde girintili olabilir.
- Girinti unutulsa bile `end` (döngülerde `do`) bloğu kapatır.
- Değişken bağlama: `ad tip => ifade` veya `ad => ifade`.
- Mantıksal/karşılaştırma operatörleri hem sembolik (`==`, `!=`, `<`, `>`,
  `&&`, `||`) hem de kelime biçiminde (`equal`, `not equal`, `less`, `greater`,
  `and`, `or`, `not`) yazılabilir.
- `fncall.ad(...)` üst düzey fonksiyon çağrısı için eşanlamlı sözdizimidir.
- Yerleşikler: `stdout`, `stderr`, `stdin`, `system.exec`.
  - `system.exec(cmd, output=True)` komutun çıktısını string olarak döndürür.
  - `system.exec(cmd, output=False)` yalnızca komutu çalıştırır.

## Şu an desteklenenler

- `import`, `fn`, `class` (alan düzeyinde), parametreler, dönüş değerleri.
- `int`, `uint`, `long`, `ulong`, `short`, `ushort`, `char`, `uchar`, `bool`,
  `string`, `void`, sabit boyutlu diziler.
- `if` / `else`, `loop` / `do`, `while`, `break`, `continue`, `return`.
- Aritmetik, karşılaştırma, mantıksal, bit düzeyi ve atama operatörleri.
- Dizi sınır kontrolü, otomatik sıfır başlatma.
- Derleme zamanı "fat pointer" izleme: `p ptr int => dizi` sonrasında `p[i]`
  erişimi dizinin bilinen uzunluğuna göre sınır kontrolünden geçer.
- `own` belleğinde `alloc`/`free` sahiplik denetimi: use-after-free, double-free
  ve sahipsiz `free` derleme zamanında reddedilir.
- SysV x86-64 ABI: 6'dan fazla argüman stack üzerinden aktarılır.
- Doğrudan SysV x86-64 kod üretimi ve libc bağlantısı.

## Bellek modelleri

```bash
target/release/fallenpp build dosya.fpp --memory=stack   # en hızlı
target/release/fallenpp build dosya.fpp --memory=safe    # null/bölme/sınır
target/release/fallenpp build dosya.fpp --memory=own     # + sahiplik analizi
```

Bayrak verilmezse ve terminal etkileşimliyse derleyici seçim sorar; boru/CI
ortamlarında `stack` varsayılanı kullanılır.

## Yol haritası

- `float`/`double` (SSE), aggregate dönüş/parametre.
- Çalışma zamanı uzunluk taşıyan tam fat pointer (fonksiyon geçişlerinde koruma).
- Borrow checker, otomatik `drop`, kapsam sonu temizlik.
- Kalıtım, sanal metotlar, destructor, fonksiyon pointer'ları.
- Gerçek modül sistemi ve standart kütüphane.
- Satır/kolon bazlı tanılamalar, AST/IR çıktıları, fuzz ve golden testler.

## Testler

```bash
./tests/run.sh
```

Testler; temel çıktı, fonksiyon çağrıları, `fncall`, girintisiz `end`
kullanımı, `stdin`/`stdout`, `system.exec` çıktı yakalama ve döngü/shell
akışını doğrular.

## Lisans

Bu proje [MIT Lisansı](LICENSE) altında lisanslanmıştır.
