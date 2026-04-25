// Log sistemi
const logSatirlar = [];
const MAX_LOG_SATIR = 300;

function logYaz(mesaj, tip) {
    const satir = { mesaj: mesaj || '', tip: tip || '' };
    logSatirlar.push(satir);
    if (logSatirlar.length > MAX_LOG_SATIR) {
        logSatirlar.shift();
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
});
