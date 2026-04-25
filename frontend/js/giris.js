let girisAktif = false;

async function girisBaslat() {
    if (girisAktif) return;

    const kullanici = document.getElementById('kullanici').value.trim();
    const sifre = document.getElementById('sifre').value;

    if (!kullanici || !sifre) {
        await modalUyari('Eksik Bilgi', 'Kullanıcı adı ve şifre gerekli.');
        return;
    }

    girisAktif = true;
    const btn = document.getElementById('giris-btn');
    btn.disabled = true;
    btn.textContent = 'Bağlanıyor...';
    durumGuncelle('Bağlanıyor...', 'yukle');

    try {
        const tc = await invoke('tc_maskele', { tc: kullanici });
        logYaz('');
        logYaz('━━━ Giriş başlatılıyor ━━━', 'bilgi');
        logYaz('  Kullanıcı: ' + tc, 'soluk');
    } catch (_) {}

    try {
        const sonuc = await invoke('giris', {
            url: GIRIS_URL,
            kullanici: kullanici,
            sifre: sifre,
        });

        logYaz('✓ Bağlantı başarılı!', 'basarili');
        logYaz('  Kullanıcı: ' + sonuc.bilgi.isim, 'soluk');
        if (sonuc.ip) logYaz('  Sunucu: ' + sonuc.ip, 'soluk');
        if (sonuc.kayit_hatasi) {
            logYaz('Kayıt uyarısı: ' + sonuc.kayit_hatasi, 'uyari');
        } else {
            await profilleriYukle({ sessiz: true });
        }

        const kota = sonuc.bilgi.kota || {};
        if (kota.kalan_mb && kota.toplam_mb) {
            try {
                const kalanGb = (parseFloat(kota.kalan_mb) / 1024).toFixed(1);
                const toplamGb = (parseFloat(kota.toplam_mb) / 1024).toFixed(1);
                logYaz('  Kota: ' + kalanGb + ' / ' + toplamGb + ' GB', 'soluk');
            } catch (_) {}
        }
        if (sonuc.bilgi.konum) logYaz('  Konum: ' + sonuc.bilgi.konum, 'soluk');

        durumGuncelle('Bağlı', 'basari');
        setTimeout(function() {
            hosgeldinGoster(sonuc.bilgi);
            yeniVersiyonKontrolEt();
        }, 600);

    } catch (hataJson) {
        try {
            const hata = JSON.parse(hataJson);
            await hataIsle(hata, kullanici, sifre);
        } catch (_) {
            logYaz('✗ Hata: ' + hataJson, 'hata');
            durumGuncelle('Hata', 'hata');
            await modalUyari('Hata', String(hataJson));
        }
    } finally {
        girisAktif = false;
        btn.disabled = false;
        btn.textContent = 'Bağlan';
    }
}

async function hataIsle(hata, kullanici, sifre) {
    switch (hata.tip) {
        case 'MaksimumCihaz':
            await maksimumCihazSor(hata.detay.cihaz_bilgisi, kullanici, sifre);
            break;
        case 'GirisBasarisiz':
            logYaz('✗ ' + hata.detay.kullanici_mesaji, 'hata');
            durumGuncelle('Giriş Başarısız', 'hata');
            await modalUyari('Giriş Başarısız', hata.detay.kullanici_mesaji);
            break;
        case 'ZamanAsimi':
            logYaz('✗ Sunucu yanıtlamıyor', 'hata');
            durumGuncelle('Hata', 'hata');
            await modalUyari('Zaman Aşımı', 'Sunucu yanıtlamıyor. Ağ yoğunluğu nedeniyle gecikmeli olabilir.');
            break;
        default:
            var mesaj = (hata.detay && hata.detay.kullanici_mesaji) || 'Bilinmeyen hata';
            logYaz('✗ ' + mesaj, 'hata');
            durumGuncelle('Hata', 'hata');
            await modalUyari('Hata', mesaj);
    }
}

async function maksimumCihazSor(bilgi, kullanici, sifre) {
    var mesaj = 'Bu hesapla başka bir cihaz zaten bağlı.\n\n' +
        'Bağlantı Başlangıcı: ' + (bilgi.baslangic || '?') + '\n' +
        'MAC Adresi: ' + (bilgi.mac || '?') + '\n' +
        'Konum: ' + (bilgi.konum || '?') + '\n\n' +
        'Önceki cihazın bağlantısını düşürüp tekrar bağlanılsın mı?';

    var onay = await modalOnay('Maksimum Cihaz Limiti', mesaj);
    if (!onay) {
        durumGuncelle('İptal edildi', 'bekle');
        return;
    }

    durumGuncelle('Önceki oturum kapatılıyor...', 'yukle');
    logYaz('↻ Önceki cihazın bağlantısı kapatılıyor...', 'uyari');

    var btn = document.getElementById('giris-btn');
    btn.disabled = true;
    btn.textContent = 'Bağlanıyor...';

    try {
        var sonuc = await invoke('maksimum_cihaz_isle', {
            url: GIRIS_URL,
            kullanici: kullanici,
            sifre: sifre,
        });
        logYaz('✓ Bağlantı başarılı!', 'basarili');
        durumGuncelle('Bağlı', 'basari');
        if (sonuc.kayit_hatasi) {
            logYaz('Kayıt uyarısı: ' + sonuc.kayit_hatasi, 'uyari');
        } else {
            await profilleriYukle({ sessiz: true });
        }
        hosgeldinGoster(sonuc.bilgi);
        yeniVersiyonKontrolEt();
    } catch (e) {
        logYaz('✗ Önceki cihaz bağlantısı düşürülemedi', 'hata');
        durumGuncelle('Hata', 'hata');
        await modalUyari('Hata', 'Önceki cihazın bağlantısı düşürülemedi. Lütfen cihazdan manuel çıkış yapın.');
    } finally {
        btn.disabled = false;
        btn.textContent = 'Bağlan';
    }
}

// Event listeners
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('giris-btn').addEventListener('click', girisBaslat);
    document.getElementById('sifre').addEventListener('keydown', function(e) {
        if (e.key === 'Enter') girisBaslat();
    });
    document.getElementById('kullanici').addEventListener('keydown', function(e) {
        if (e.key === 'Enter') document.getElementById('sifre').focus();
    });
});
