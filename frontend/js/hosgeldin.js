// Kota bittiğinde kartı %0 kalan olarak gösterir.
function kotaKartiDoldu(kotaKart, kota, toplamMb) {
    kotaKart.classList.remove('gizli');

    var bar = document.getElementById('kota-bar');
    bar.style.width = '0%';
    bar.className = 'kota-bar-ic';
    bar.style.background = 'linear-gradient(90deg, var(--kirmizi), var(--kirmizi-acik))';

    var buyuk = document.getElementById('kota-yuzde-buyuk');
    buyuk.className = 'kota-yuzde-buyuk tip-low';
    buyuk.textContent = '0%';

    var detay = document.getElementById('kota-detay');
    if (toplamMb > 0) {
        var toplamGb = toplamMb / 1024;
        detay.innerHTML =
            '<span><strong>0.0 GB</strong> kalan</span>' +
            '<span>' + toplamGb.toFixed(0) + ' / ' + toplamGb.toFixed(0) + ' GB kullanıldı</span>';
    } else {
        detay.innerHTML = '<span><strong>Kotanız doldu</strong></span>';
    }

    var yenilenmeEl = document.getElementById('kota-yenilenme');
    if (kota.yenilenme) {
        yenilenmeEl.textContent = kota.yenilenme + ' yenilenir';
        yenilenmeEl.style.display = '';
    } else {
        yenilenmeEl.style.display = 'none';
    }
}

function hosgeldinGoster(bilgi) {
    document.getElementById('isim-lbl').textContent = bilgi.isim || '';

    // Bilgi satirlari (Konum + Son Giris) — ikon + label + value
    var bilgiKart = document.getElementById('bilgi-kart');
    if (bilgi.son_giris || bilgi.konum) {
        bilgiKart.classList.remove('gizli');
        var konumEl = document.getElementById('konum-bilgi');
        var sonGirisEl = document.getElementById('son-giris');

        if (bilgi.konum) {
            konumEl.classList.remove('gizli');
            konumEl.innerHTML =
                SVG.pin +
                '<span class="lbl">Konum</span>' +
                '<span class="val"></span>';
            konumEl.querySelector('.val').textContent = bilgi.konum;
        } else {
            konumEl.classList.add('gizli');
        }

        if (bilgi.son_giris) {
            sonGirisEl.classList.remove('gizli');
            sonGirisEl.innerHTML =
                SVG.clock +
                '<span class="lbl">Son Giriş</span>' +
                '<span class="val"></span>';
            sonGirisEl.querySelector('.val').textContent = bilgi.son_giris;
        } else {
            sonGirisEl.classList.add('gizli');
        }
    } else {
        bilgiKart.classList.add('gizli');
    }

    // Kota karti
    var kotaKart = document.getElementById('kota-kart');
    var kota = bilgi.kota || {};
    var toplamMb = parseFloat(kota.toplam_mb);
    var kalanMb = parseFloat(kota.kalan_mb);
    var kotaDoldu = bilgi.kota_doldu || (toplamMb > 0 && kalanMb === 0);

    if (kotaDoldu) {
        kotaKartiDoldu(kotaKart, kota, toplamMb);
    } else if (toplamMb > 0 && kalanMb > 0) {
        kotaKart.classList.remove('gizli');
        var oran = Math.max(0, Math.min(1, kalanMb / toplamMb));
        var tone = oran > 0.5 ? 'ok' : oran > 0.2 ? 'warn' : 'low';

        var bar = document.getElementById('kota-bar');
        bar.style.width = '0%';
        bar.className = 'kota-bar-ic';
        if (tone === 'ok') {
            bar.style.background = 'linear-gradient(90deg, var(--yesil), var(--yesil-acik))';
        } else if (tone === 'warn') {
            bar.style.background = 'linear-gradient(90deg, var(--sari), var(--sari-acik))';
        } else {
            bar.style.background = 'linear-gradient(90deg, var(--kirmizi), var(--kirmizi-acik))';
        }
        requestAnimationFrame(function() {
            bar.style.width = (oran * 100) + '%';
        });

        var buyuk = document.getElementById('kota-yuzde-buyuk');
        buyuk.className = 'kota-yuzde-buyuk tip-' + tone;
        buyuk.textContent = Math.round(oran * 100) + '%';

        var kalanGb = kalanMb / 1024;
        var toplamGb = toplamMb / 1024;
        var kullanilanGb = (toplamMb - kalanMb) / 1024;

        var detay = document.getElementById('kota-detay');
        detay.innerHTML =
            '<span><strong>' + kalanGb.toFixed(1) + ' GB</strong> kalan</span>' +
            '<span>' + kullanilanGb.toFixed(1) + ' / ' + toplamGb.toFixed(0) + ' GB kullanıldı</span>';

        var yenilenmeEl = document.getElementById('kota-yenilenme');
        if (kota.yenilenme) {
            yenilenmeEl.textContent = kota.yenilenme + ' yenilenir';
            yenilenmeEl.style.display = '';
        } else {
            yenilenmeEl.style.display = 'none';
        }
    } else {
        kotaKart.classList.add('gizli');
    }

    ekranGoster('ekran-hosgeldin');
}

document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('cikis-btn').addEventListener('click', async function() {
        logYaz('Çıkış yapılıyor…', 'uyari');

        var cikisTamam = false;

        try {
            var basarili = await invoke('cikis');
            if (basarili) {
                cikisTamam = true;
                logYaz('Oturum sonlandırıldı', 'basarili');
            } else {
                logYaz('Çıkış isteği gönderilemedi', 'uyari');
            }
        } catch (_) {
            logYaz('Çıkış isteği gönderilemedi', 'uyari');
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
