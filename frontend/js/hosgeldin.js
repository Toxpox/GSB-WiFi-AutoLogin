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

// gecisYok=true: kart yeniden doldurulur ama ekran gecisi yapilmaz (zaten
// bagli ekrandayken "Bilgileri yenile" flicker olusturmasin).
function hosgeldinGoster(bilgi, gecisYok) {
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

    kotaGecmisiCiz();
    if (!gecisYok) ekranGoster('ekran-hosgeldin');
}

// "Bilgileri yenile": yeniden giris yapmadan aktif oturumdan guncel
// kullanici/kota bilgisini ceker ve karti (grafik dahil) gunceller.
async function bilgileriYenile() {
    var btn = document.getElementById('bilgi-yenile-btn');
    if (btn) { btn.classList.add('donuyor'); btn.disabled = true; }
    try {
        var bilgi = await invoke('bilgi_yenile');
        hosgeldinGoster(bilgi, true);
        logYaz('Bilgiler yenilendi.', 'soluk');
    } catch (e) {
        logYaz('Bilgiler yenilenemedi: ' + String(e), 'uyari');
        await modalUyari('Yenilenemedi', String(e));
    } finally {
        if (btn) { btn.classList.remove('donuyor'); btn.disabled = false; }
    }
}

// Kota gecmisinden inline SVG sparkline + tukenme tahmini cizer. En az 2 gunluk
// veri yoksa (tek nokta egri cizmez) gizli kalir. Harici grafik kutuphanesi yok.
async function kotaGecmisiCiz() {
    var grafik = document.getElementById('kota-grafik');
    var cizim = document.getElementById('kota-grafik-cizim');
    var tahminEl = document.getElementById('kota-tahmin');
    if (!grafik || !cizim) return;

    var gecmis = [];
    try { gecmis = await invoke('kota_gecmisi_al'); } catch (_) {}
    if (!Array.isArray(gecmis)) gecmis = [];
    gecmis = gecmis.slice(-30);

    var toplam = 0;
    gecmis.forEach(function(k) { if (k.toplam_mb > toplam) toplam = k.toplam_mb; });

    // Yeterli veri yoksa karti gizlemek yerine bilgilendirici metin goster;
    // boylece kullanici ozelligin var oldugunu ve verinin biriktigini bilir
    // (grafik en az 2 farkli gunluk veriyle cizilir).
    if (gecmis.length < 2 || toplam <= 0) {
        if (tahminEl) tahminEl.textContent = '';
        cizim.innerHTML = '<div class="kota-grafik-bos">Grafik birkaç günlük kullanım verisiyle oluşur.</div>';
        grafik.classList.remove('gizli');
        return;
    }

    // Sparkline: kalan_mb degerini [0..toplam] araliginda normalize eder.
    var n = gecmis.length;
    var noktalar = gecmis.map(function(k, i) {
        var x = n > 1 ? (i / (n - 1)) * 100 : 0;
        var oran = Math.max(0, Math.min(1, k.kalan_mb / toplam));
        var y = 34 - oran * 30 + 1; // ust/alt 2px pay
        return x.toFixed(2) + ',' + y.toFixed(2);
    });
    var cizgi = noktalar.join(' ');
    var alan = cizgi + ' 100,36 0,36';
    cizim.innerHTML =
        '<svg class="kota-spark" viewBox="0 0 100 36" preserveAspectRatio="none">' +
            '<polygon points="' + alan + '" fill="currentColor" fill-opacity="0.12"/>' +
            '<polyline points="' + cizgi + '" fill="none" stroke="currentColor" stroke-width="1.6" ' +
                'vector-effect="non-scaling-stroke" stroke-linejoin="round" stroke-linecap="round"/>' +
        '</svg>';

    // Tahmin: son yenilenmeden (kalan artisindan) bu yana olan segment uzerinden
    // gunluk tuketim ortalamasi -> kalan / gunluk = kac gun sonra biter.
    var basla = 0;
    for (var i = 1; i < gecmis.length; i++) {
        if (gecmis[i].kalan_mb > gecmis[i - 1].kalan_mb + 1) basla = i; // 1 MB tolerans
    }
    var seg = gecmis.slice(basla);
    var tahmin = '';
    if (seg.length >= 2) {
        var ilk = seg[0], son = seg[seg.length - 1];
        var gunSpan = Math.max(1, Math.round((Date.parse(son.tarih) - Date.parse(ilk.tarih)) / 86400000));
        var dususMb = ilk.kalan_mb - son.kalan_mb;
        if (dususMb > 0) {
            var gunlukMb = dususMb / gunSpan;
            var gunKaldi = Math.floor(son.kalan_mb / gunlukMb);
            tahmin = isFinite(gunKaldi) ? ('Bu hızla ~' + gunKaldi + ' gün sonra biter') : '';
        } else {
            tahmin = 'Kullanım düşük';
        }
    }
    if (tahminEl) tahminEl.textContent = tahmin;
    grafik.classList.remove('gizli');
}

// Tepsi menusunden de cagrilir (app.js: tepsiOlaylariniDinle).
async function cikisYap() {
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
        await modalUyari('Çıkış Başarısız', 'Çıkış isteği tamamlanamadı. GSB WiFi ağına bağlı olduğunuzdan emin olun.');
        return;
    }

    ekranGoster('ekran-giris');
    durumGuncelle('Hazır', 'bekle');
    document.getElementById('sifre').value = '';
}

document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('cikis-btn').addEventListener('click', cikisYap);
    var ybtn = document.getElementById('bilgi-yenile-btn');
    if (ybtn) ybtn.addEventListener('click', bilgileriYenile);
});
