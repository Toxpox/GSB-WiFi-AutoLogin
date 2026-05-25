const { invoke } = window.__TAURI__.core;

let VERSION = "1.7.0";
let GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check";
let KAYITLI_PROFILLER = [];
let SECILI_PROFIL_ID = null;
let VERSIYON_KONTROL_EDILDI = false;

// --- Inline SVG snippets used by JS-rendered nodes ---
const SVG = {
    close: '<svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12"/><path d="M18 6L6 18"/></svg>',
    eye: '<path d="M2.5 12s3.5-6.5 9.5-6.5S21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Z"/><circle cx="12" cy="12" r="2.6"/>',
    eyeOff: '<path d="M3 3l18 18"/><path d="M10.6 6.1A9.8 9.8 0 0 1 12 6c6 0 9.5 6 9.5 6a16.6 16.6 0 0 1-3.3 3.9"/><path d="M6.1 7.4A16.4 16.4 0 0 0 2.5 12s3.5 6 9.5 6a9.6 9.6 0 0 0 4.2-1"/><path d="M9.5 9.7a3 3 0 0 0 4.2 4.2"/>',
    pin: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M12 21s7-6.2 7-12a7 7 0 1 0-14 0c0 5.8 7 12 7 12Z"/><circle cx="12" cy="9.5" r="2.5"/></svg>',
    clock: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 7.5V12l3 2"/></svg>',
    modalIkon: {
        hata:    '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12"/><path d="M18 6L6 18"/></svg>',
        uyari:   '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3.5 22 20H2L12 3.5Z"/><path d="M12 10v5"/><circle cx="12" cy="17.5" r="0.8" fill="currentColor" stroke="none"/></svg>',
        bilgi:   '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 11v6"/><circle cx="12" cy="7.8" r="0.8" fill="currentColor" stroke="none"/></svg>',
        soru:    '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M9.2 9.3a3 3 0 1 1 4.4 2.6c-1.1.6-1.6 1.2-1.6 2.5"/><circle cx="12" cy="17" r="0.9" fill="currentColor" stroke="none"/></svg>',
        basari:  '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12.5l5 5L20 6.5"/></svg>',
    }
};

// Ekran yonetimi
function ekranGoster(id) {
    document.querySelectorAll('.ekran').forEach(e => {
        e.classList.remove('aktif');
        e.classList.add('gizli');
    });
    const hedef = document.getElementById(id);
    requestAnimationFrame(() => {
        hedef.classList.remove('gizli');
        hedef.classList.add('aktif');
    });
}

// Durum cizgisi (login ekrani altinda)
function durumGuncelle(metin, tip) {
    const cizgi = document.getElementById('durum');
    if (!cizgi) return;
    cizgi.className = 'statusline durum-' + (tip || 'bekle');
    const yazi = cizgi.querySelector('.durum-yazi');
    if (yazi) yazi.textContent = metin;
}

async function baslatmaYukle() {
    try {
        const appBilgisi = await invoke('app_bilgisi');
        VERSION = appBilgisi.version || VERSION;
        GIRIS_URL = appBilgisi.giris_url || GIRIS_URL;
    } catch (_) {}

    await profilleriYukle({ sessiz: true });

    logYaz("GSB WiFi AutoLogin v" + VERSION, "bilgi");
    logYaz("Hazır.", "soluk");
}

async function profilleriYukle(secenekler) {
    const opts = secenekler || {};
    try {
        KAYITLI_PROFILLER = await invoke('profilleri_listele');
    } catch (_) {
        KAYITLI_PROFILLER = [];
        try {
            const [kullanici, sifre] = await invoke('kayitli_kullanici');
            if (kullanici) document.getElementById('kullanici').value = kullanici;
            if (sifre) document.getElementById('sifre').value = sifre;
        } catch (_) {}
    }

    profilListesiniCiz();

    if (KAYITLI_PROFILLER.length === 0) {
        SECILI_PROFIL_ID = null;
        return;
    }

    var hedef = KAYITLI_PROFILLER.find(function(p) { return p.aktif; }) || KAYITLI_PROFILLER[0];
    if (hedef) {
        await profilSec(hedef.id, { sessiz: opts.sessiz !== false });
    }
}

function profilBasHarfleri(label) {
    if (!label) return '??';
    var m = String(label).match(/^\d+/);
    return (m ? m[0] : String(label).replace(/\W/g, '')).slice(0, 2).toUpperCase() || '??';
}

function profilListesiniCiz() {
    const liste = document.getElementById('profil-listesi');
    const bos = document.getElementById('profil-bos');
    if (!liste || !bos) return;

    liste.textContent = '';
    bos.classList.toggle('gizli', KAYITLI_PROFILLER.length > 0);

    const panel = liste.closest('.profil-panel');
    if (panel) panel.classList.toggle('gizli', KAYITLI_PROFILLER.length === 0);

    KAYITLI_PROFILLER.forEach(function(profil) {
        const aktif = profil.id === SECILI_PROFIL_ID || profil.aktif;
        const token = document.createElement('div');
        token.className = 'token' + (aktif ? ' aktif' : '');

        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'token-btn';
        btn.title = profil.masked_username || 'Profil';
        btn.addEventListener('click', function() { profilSec(profil.id); });

        const avatar = document.createElement('span');
        avatar.className = 'token-avatar';
        avatar.textContent = profilBasHarfleri(profil.masked_username);
        btn.appendChild(avatar);

        if (aktif) {
            const name = document.createElement('span');
            name.className = 'token-name';
            name.textContent = profil.masked_username || 'Profil';
            btn.appendChild(name);
        }

        const del = document.createElement('button');
        del.type = 'button';
        del.className = 'token-del';
        del.title = 'Profili sil';
        del.setAttribute('aria-label', 'Profili sil');
        del.innerHTML = SVG.close;
        del.addEventListener('click', function(e) {
            e.stopPropagation();
            profilSil(profil.id, profil.masked_username || 'Profil');
        });

        token.appendChild(btn);
        token.appendChild(del);
        liste.appendChild(token);
    });
}

async function profilSec(id, secenekler) {
    const opts = secenekler || {};
    try {
        const [kullanici, sifre] = await invoke('profil_yukle', { id: id });
        SECILI_PROFIL_ID = id;
        document.getElementById('kullanici').value = kullanici || '';
        document.getElementById('sifre').value = sifre || '';
        KAYITLI_PROFILLER.forEach(function(p) { p.aktif = p.id === id; });
        profilListesiniCiz();
        if (!opts.sessiz) {
            const profil = KAYITLI_PROFILLER.find(function(p) { return p.id === id; });
            logYaz('Profil seçildi: ' + ((profil && profil.masked_username) || kullanici), 'soluk');
        }
    } catch (e) {
        if (!opts.sessiz) {
            await modalUyari('Profil Yüklenemedi', String(e));
        }
    }
}

async function profilSil(id, ad) {
    const onay = await modalOnay('Profil Silinsin mi?', ad + ' profili bu cihazdan silinecek.');
    if (!onay) return;

    try {
        KAYITLI_PROFILLER = await invoke('profil_sil', { id: id });
        if (SECILI_PROFIL_ID === id) {
            SECILI_PROFIL_ID = null;
            document.getElementById('kullanici').value = '';
            document.getElementById('sifre').value = '';
        }
        profilListesiniCiz();

        const aktif = KAYITLI_PROFILLER.find(function(p) { return p.aktif; });
        if (aktif) {
            await profilSec(aktif.id, { sessiz: true });
        }
        logYaz('Profil silindi: ' + ad, 'uyari');
    } catch (e) {
        await modalUyari('Profil Silinemedi', String(e));
    }
}

function profilSeciminiTemizle() {
    if (!SECILI_PROFIL_ID) return;
    SECILI_PROFIL_ID = null;
    KAYITLI_PROFILLER.forEach(function(p) { p.aktif = false; });
    profilListesiniCiz();
}

function sifreGorunurlukDegistir() {
    const sifre = document.getElementById('sifre');
    const btn = document.getElementById('sifre-goster-btn');
    const yazi = btn.querySelector('.sifre-goster-yazi');
    const svg = btn.querySelector('.ico-eye');
    const wasVisible = sifre.type === 'text';
    sifre.type = wasVisible ? 'password' : 'text';
    btn.title = wasVisible ? 'Şifreyi göster' : 'Şifreyi gizle';
    btn.setAttribute('aria-label', btn.title);
    btn.classList.toggle('aktif', !wasVisible);
    if (yazi) yazi.textContent = wasVisible ? 'Göster' : 'Gizle';
    if (svg) svg.innerHTML = wasVisible ? SVG.eye : SVG.eyeOff;
}

async function githubAc() {
    try {
        await invoke('github_ac');
        logYaz('GitHub deposu açılıyor.', 'soluk');
    } catch (e) {
        await modalUyari('GitHub Açılamadı', String(e));
    }
}

async function yeniVersiyonKontrolEt() {
    if (VERSIYON_KONTROL_EDILDI) return;
    VERSIYON_KONTROL_EDILDI = true;

    try {
        const sonuc = await invoke('yeni_versiyon_kontrol');
        if (sonuc.guncel || !sonuc.release_url) return;

        const son = sonuc.son || 'yeni sürüm';
        logYaz('Yeni sürüm mevcut: ' + son, 'uyari');
        const acilsin = await modalOnay(
            'Yeni Sürüm Var',
            'Mevcut sürüm: ' + sonuc.mevcut + '\nYeni sürüm: ' + son + '\n\nGitHub release sayfası açılsın mı?'
        );
        if (acilsin) {
            await invoke('github_link_ac', { url: sonuc.release_url });
        }
    } catch (e) {
        logYaz('Sürüm kontrolü yapılamadı: ' + String(e), 'uyari');
    }
}

// --- Modal Dialog ---
function modalGoster(baslik, mesaj, tip, butonlar) {
    var overlay = document.getElementById('modal-overlay');
    var ikonEl = document.getElementById('modal-ikon');
    var baslikEl = document.getElementById('modal-baslik');
    var mesajEl = document.getElementById('modal-mesaj');
    var btnAlani = document.getElementById('modal-butonlar');

    ikonEl.className = 'modal-ikon tip-' + (tip || 'bilgi');
    ikonEl.innerHTML = SVG.modalIkon[tip] || SVG.modalIkon.bilgi;
    baslikEl.textContent = baslik;
    mesajEl.textContent = mesaj;

    btnAlani.textContent = '';
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

document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('.js-github-btn').forEach(function(btn) {
        btn.addEventListener('click', githubAc);
    });
    document.getElementById('profil-yenile-btn').addEventListener('click', function() {
        profilleriYukle({ sessiz: false });
    });
    document.getElementById('sifre-goster-btn').addEventListener('click', sifreGorunurlukDegistir);
    document.getElementById('kullanici').addEventListener('input', profilSeciminiTemizle);
    document.getElementById('sifre').addEventListener('input', profilSeciminiTemizle);
    baslatmaYukle();
});
