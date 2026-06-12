// Log sistemi
const logSatirlar = [];
const MAX_LOG_SATIR = 300;

// sadeceUi=true: satir dosyaya YAZILMAZ (backend kaynakli olaylar dosyaya
// backend tarafindan yazilir; cift kayit olusmasin).
function logYaz(mesaj, tip, sadeceUi) {
    const satir = { mesaj: mesaj || '', tip: tip || '' };
    logSatirlar.push(satir);
    if (logSatirlar.length > MAX_LOG_SATIR) {
        logSatirlar.shift();
    }

    if (!sadeceUi) {
        // Dosyaya yazim arka planda; hata UI'yi asla bloklamaz.
        try {
            invoke('log_satiri_yaz', { satir: satir.mesaj, tip: satir.tip || null })
                .catch(function() {});
        } catch (_) {}
    }

    const alan = document.getElementById('log-icerik');
    if (!alan) return;
    if (alan.children.length >= MAX_LOG_SATIR && alan.firstElementChild) {
        alan.removeChild(alan.firstElementChild);
    }
    alan.appendChild(logSatiriOlustur(satir));
    alan.scrollTop = alan.scrollHeight;
}

function logSatiriOlustur(s) {
    const el = document.createElement('div');
    el.className = 'log-satir' + (s.tip ? ' ' + s.tip : '');
    el.textContent = s.mesaj;
    return el;
}

function logPenceresiGuncelle() {
    const alan = document.getElementById('log-icerik');
    if (!alan) return;
    alan.textContent = '';
    logSatirlar.forEach(function(s) {
        alan.appendChild(logSatiriOlustur(s));
    });
    alan.scrollTop = alan.scrollHeight;
}

function logPanelAcKapa() {
    document.getElementById('log-panel').classList.toggle('acik');
}

function logTemizle() {
    logSatirlar.length = 0;
    logPenceresiGuncelle();
}

// Event listeners
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('log-btn-giris').addEventListener('click', logPanelAcKapa);
    document.getElementById('log-btn-hosgeldin').addEventListener('click', logPanelAcKapa);
    document.getElementById('log-kapat').addEventListener('click', logPanelAcKapa);
    document.getElementById('log-temizle').addEventListener('click', logTemizle);
    document.getElementById('log-klasor').addEventListener('click', function() {
        invoke('log_klasoru_ac').catch(function(e) {
            logYaz('Log klasörü açılamadı: ' + String(e), 'uyari');
        });
    });
});
