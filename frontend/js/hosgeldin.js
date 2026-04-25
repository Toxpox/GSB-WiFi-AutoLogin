function hosgeldinGoster(bilgi) {
    document.getElementById('isim-lbl').textContent = bilgi.isim;

    // Bilgi karti
    var bilgiKart = document.getElementById('bilgi-kart');
    if (bilgi.son_giris || bilgi.konum) {
        bilgiKart.classList.remove('gizli');
        document.getElementById('son-giris').textContent =
            bilgi.son_giris ? 'Son Giriş:  ' + bilgi.son_giris : '';
        document.getElementById('konum-bilgi').textContent =
            bilgi.konum ? 'Konum:  ' + bilgi.konum : '';
    } else {
        bilgiKart.classList.add('gizli');
    }

    // Kota karti
    var kotaKart = document.getElementById('kota-kart');
    var kota = bilgi.kota || {};
    var toplamMb = parseFloat(kota.toplam_mb);
    var kalanMb = parseFloat(kota.kalan_mb);

    if (toplamMb && kalanMb) {
        kotaKart.classList.remove('gizli');
        var oran = kalanMb / toplamMb;
        var bar = document.getElementById('kota-bar');

        // Animasyonlu genislik
        bar.style.width = '0%';
        requestAnimationFrame(function() {
            bar.style.width = (oran * 100) + '%';
        });

        // Renk
        if (oran > 0.5)
            bar.style.background = 'linear-gradient(90deg, var(--yesil), #34d399)';
        else if (oran > 0.2)
            bar.style.background = 'linear-gradient(90deg, var(--sari), #fbbf24)';
        else
            bar.style.background = 'linear-gradient(90deg, var(--kirmizi), #f87171)';

        var kalanGb = (kalanMb / 1024).toFixed(1);
        var toplamGb = (toplamMb / 1024).toFixed(1);
        document.getElementById('kota-yuzde').textContent =
            kalanGb + ' GB / ' + toplamGb + ' GB  (' + Math.round(oran * 100) + '%)';

        var detay = [];
        if (kota.yenilenme) detay.push('Yenilenme: ' + kota.yenilenme);
        detay.push('Kullanılan: ' + ((toplamMb - kalanMb) / 1024).toFixed(1) + ' GB');
        document.getElementById('kota-detay').textContent = detay.join('\n');
    } else {
        kotaKart.classList.add('gizli');
    }

    // Ekran gecisi
    ekranGoster('ekran-hosgeldin');
}

// Cikis butonu
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('cikis-btn').addEventListener('click', async function() {
        logYaz('');
        logYaz('━━━ Çıkış yapılıyor... ━━━', 'uyari');

        var cikisTamam = false;

        try {
            var basarili = await invoke('cikis');
            if (basarili) {
                cikisTamam = true;
                logYaz('✓ Oturum sonlandırıldı', 'basarili');
            } else {
                logYaz('⚠ Çıkış isteği gönderilemedi', 'uyari');
            }
        } catch (_) {
            logYaz('⚠ Çıkış isteği gönderilemedi', 'uyari');
        }

        if (!cikisTamam) {
            durumGuncelle('Bağlı', 'basari');
            return;
        }

        ekranGoster('ekran-giris');
        durumGuncelle('Hazır', 'bekle');
        document.getElementById('sifre').value = '';
    });
});
