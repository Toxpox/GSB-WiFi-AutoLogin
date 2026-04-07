// Log sistemi
const logSatirlar = [];

function logYaz(mesaj, tip) {
    logSatirlar.push({ mesaj: mesaj, tip: tip || '' });
    logPenceresiGuncelle();
}

function logPenceresiGuncelle() {
    const alan = document.getElementById('log-icerik');
    if (!alan) return;
    alan.innerHTML = logSatirlar.map(function(s) {
        const cls = s.tip ? ' ' + s.tip : '';
        const escaped = s.mesaj
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
        return '<div class="log-satir' + cls + '">' + escaped + '</div>';
    }).join('');
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
