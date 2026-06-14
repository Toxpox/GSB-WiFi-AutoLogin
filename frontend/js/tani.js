// Baglanti tanilama paneli — asama asama self-test sonuclarini gosterir.
// Sessiz otomatik-giris hatalarini (yanlis ag, DNS, kapali port, portal
// degisikligi) somut teshise cevirir. Backend: tani_calistir komutu.

function taniPanelAcKapa() {
    var panel = document.getElementById('tani-panel');
    if (!panel) return;
    // Ayni anda tek alt-panel acik kalsin.
    var logPanel = document.getElementById('log-panel');
    if (logPanel) logPanel.classList.remove('acik');
    var acildi = !panel.classList.contains('acik');
    panel.classList.toggle('acik');
    if (acildi) taniCalistir();
}

function taniDurumYazi(durum) {
    switch (durum) {
        case 'ok': return 'Başarılı';
        case 'uyari': return 'Uyarı';
        case 'hata': return 'Hata';
        default: return 'Bilgi';
    }
}

function taniSatiriOlustur(s) {
    var durum = s.durum || 'bilgi';
    var satir = document.createElement('div');
    satir.className = 'tani-satir ' + durum;

    var dot = document.createElement('span');
    dot.className = 'tani-dot';

    var govde = document.createElement('div');
    govde.className = 'tani-govde';

    var ust = document.createElement('div');
    ust.className = 'tani-ust';
    var ad = document.createElement('span');
    ad.className = 'tani-ad';
    ad.textContent = s.ad || '';
    var rozet = document.createElement('span');
    rozet.className = 'tani-rozet';
    rozet.textContent = taniDurumYazi(durum);
    ust.appendChild(ad);
    ust.appendChild(rozet);

    var detay = document.createElement('div');
    detay.className = 'tani-detay';
    detay.textContent = s.detay || '';

    govde.appendChild(ust);
    govde.appendChild(detay);
    satir.appendChild(dot);
    satir.appendChild(govde);
    return satir;
}

async function taniCalistir() {
    var alan = document.getElementById('tani-icerik');
    var btn = document.getElementById('tani-calistir');
    if (!alan) return;
    if (btn) { btn.disabled = true; btn.textContent = 'Çalışıyor…'; }

    alan.textContent = '';
    var yuk = document.createElement('div');
    yuk.className = 'tani-yukleniyor';
    yuk.textContent = 'Tanılama çalışıyor…';
    alan.appendChild(yuk);

    try {
        var sonuclar = await invoke('tani_calistir');
        alan.textContent = '';
        (sonuclar || []).forEach(function(s) {
            alan.appendChild(taniSatiriOlustur(s));
        });
        if (!sonuclar || sonuclar.length === 0) {
            alan.textContent = 'Sonuç alınamadı.';
        }
    } catch (e) {
        alan.textContent = '';
        alan.appendChild(taniSatiriOlustur({
            ad: 'Tanılama', durum: 'hata', detay: 'Tanılama çalıştırılamadı: ' + String(e)
        }));
        try { logYaz('Tanılama başarısız: ' + String(e), 'hata'); } catch (_) {}
    } finally {
        if (btn) { btn.disabled = false; btn.textContent = 'Çalıştır'; }
    }
}

document.addEventListener('DOMContentLoaded', function() {
    var g = document.getElementById('tani-btn-giris');
    var h = document.getElementById('tani-btn-hosgeldin');
    if (g) g.addEventListener('click', taniPanelAcKapa);
    if (h) h.addEventListener('click', taniPanelAcKapa);
    var kapat = document.getElementById('tani-kapat');
    if (kapat) kapat.addEventListener('click', taniPanelAcKapa);
    var calistir = document.getElementById('tani-calistir');
    if (calistir) calistir.addEventListener('click', taniCalistir);
});
