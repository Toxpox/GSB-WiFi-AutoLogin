const { invoke } = window.__TAURI__.core;

const VERSION = "1.5.0";
const GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check";

// Ekran yonetimi
function ekranGoster(id) {
    document.querySelectorAll('.ekran').forEach(e => {
        e.classList.remove('aktif');
        e.classList.add('gizli');
    });
    const hedef = document.getElementById(id);
    // Animasyon icin kisa gecikme
    requestAnimationFrame(() => {
        hedef.classList.remove('gizli');
        hedef.classList.add('aktif');
    });
}

// Durum rozeti guncelle
function durumGuncelle(metin, tip) {
    const rozet = document.getElementById('durum');
    rozet.className = 'durum-rozeti durum-' + tip;
    rozet.querySelector('.durum-yazi').textContent = metin;
}

// Baslangicta kayitli kullanici yukle
async function baslatmaYukle() {
    try {
        const [kullanici, sifre] = await invoke('kayitli_kullanici');
        if (kullanici) document.getElementById('kullanici').value = kullanici;
        if (sifre) document.getElementById('sifre').value = sifre;
    } catch (_) {}

    logYaz("┌───────────────────────────┐", "bilgi");
    logYaz("│  GSB WiFi AutoLogin v" + VERSION + "          │", "bilgi");
    logYaz("└───────────────────────────┘", "bilgi");
    logYaz("");
}

// --- Modal Dialog Sistemi ---
function modalGoster(baslik, mesaj, tip, butonlar) {
    var overlay = document.getElementById('modal-overlay');
    var ikonEl = document.getElementById('modal-ikon');
    var baslikEl = document.getElementById('modal-baslik');
    var mesajEl = document.getElementById('modal-mesaj');
    var btnAlani = document.getElementById('modal-butonlar');

    // Ikon
    var ikonlar = {
        hata: '&#x26D4;',
        uyari: '&#x26A0;&#xFE0F;',
        bilgi: '&#x2139;&#xFE0F;',
        soru: '&#x2753;',
        basari: '&#x2705;'
    };
    ikonEl.innerHTML = ikonlar[tip] || ikonlar.bilgi;
    baslikEl.textContent = baslik;
    mesajEl.textContent = mesaj;

    // Butonlar
    btnAlani.innerHTML = '';
    return new Promise(function(resolve) {
        butonlar.forEach(function(btn) {
            var el = document.createElement('button');
            el.textContent = btn.text;
            el.className = btn.cls || 'modal-btn-ana';
            el.addEventListener('click', function() {
                overlay.classList.add('gizli');
                resolve(btn.value);
            });
            btnAlani.appendChild(el);
        });
        overlay.classList.remove('gizli');
    });
}

function modalUyari(baslik, mesaj) {
    return modalGoster(baslik, mesaj, 'hata', [
        { text: 'Tamam', cls: 'modal-btn-ana', value: true }
    ]);
}

function modalOnay(baslik, mesaj) {
    return modalGoster(baslik, mesaj, 'soru', [
        { text: 'Hayır', cls: 'modal-btn-iptal', value: false },
        { text: 'Evet', cls: 'modal-btn-ana', value: true }
    ]);
}

function modalBilgi(baslik, mesaj) {
    return modalGoster(baslik, mesaj, 'bilgi', [
        { text: 'Tamam', cls: 'modal-btn-ana', value: true }
    ]);
}

// Sayfa yuklendikten sonra baslat
document.addEventListener('DOMContentLoaded', baslatmaYukle);
