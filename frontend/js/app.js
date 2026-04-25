const { invoke } = window.__TAURI__.core;

let VERSION = "1.6.1";
let GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check";
let KAYITLI_PROFILLER = [];
let SECILI_PROFIL_ID = null;
let VERSIYON_KONTROL_EDILDI = false;

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
        const appBilgisi = await invoke('app_bilgisi');
        VERSION = appBilgisi.version || VERSION;
        GIRIS_URL = appBilgisi.giris_url || GIRIS_URL;
    } catch (_) {}

    await profilleriYukle({ sessiz: true });

    logYaz("┌───────────────────────────┐", "bilgi");
    logYaz("│  GSB WiFi AutoLogin v" + VERSION + "          │", "bilgi");
    logYaz("└───────────────────────────┘", "bilgi");
    logYaz("");
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

function profilListesiniCiz() {
    const liste = document.getElementById('profil-listesi');
    const bos = document.getElementById('profil-bos');
    if (!liste || !bos) return;

    liste.textContent = '';
    bos.classList.toggle('gizli', KAYITLI_PROFILLER.length > 0);

    KAYITLI_PROFILLER.forEach(function(profil) {
        const satir = document.createElement('div');
        satir.className = 'profil-satir' + (profil.id === SECILI_PROFIL_ID || profil.aktif ? ' aktif' : '');

        const secBtn = document.createElement('button');
        secBtn.type = 'button';
        secBtn.className = 'profil-sec-btn';
        secBtn.title = profil.masked_username || 'Profil';
        secBtn.addEventListener('click', function() {
            profilSec(profil.id);
        });

        const ad = document.createElement('span');
        ad.className = 'profil-ad';
        ad.textContent = profil.masked_username || 'Profil';
        secBtn.appendChild(ad);

        if (profil.id === SECILI_PROFIL_ID || profil.aktif) {
            const aktif = document.createElement('span');
            aktif.className = 'profil-aktif-etiket';
            aktif.textContent = 'Aktif';
            secBtn.appendChild(aktif);
        }

        const silBtn = document.createElement('button');
        silBtn.type = 'button';
        silBtn.className = 'profil-sil-btn';
        silBtn.title = 'Profili sil';
        silBtn.setAttribute('aria-label', 'Profili sil');
        silBtn.textContent = '\u00d7';
        silBtn.addEventListener('click', function() {
            profilSil(profil.id, profil.masked_username || 'Profil');
        });

        satir.appendChild(secBtn);
        satir.appendChild(silBtn);
        liste.appendChild(satir);
    });
}

async function profilSec(id, secenekler) {
    const opts = secenekler || {};
    try {
        const [kullanici, sifre] = await invoke('profil_yukle', { id: id });
        SECILI_PROFIL_ID = id;
        document.getElementById('kullanici').value = kullanici || '';
        document.getElementById('sifre').value = sifre || '';
        KAYITLI_PROFILLER.forEach(function(p) {
            p.aktif = p.id === id;
        });
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
    KAYITLI_PROFILLER.forEach(function(p) {
        p.aktif = false;
    });
    profilListesiniCiz();
}

function sifreGorunurlukDegistir() {
    const sifre = document.getElementById('sifre');
    const btn = document.getElementById('sifre-goster-btn');
    const gorunur = sifre.type === 'text';
    sifre.type = gorunur ? 'password' : 'text';
    btn.title = gorunur ? 'Şifreyi göster' : 'Şifreyi gizle';
    btn.setAttribute('aria-label', btn.title);
    btn.classList.toggle('aktif', !gorunur);
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
        if (sonuc.guncel || !sonuc.release_url) {
            return;
        }

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

// --- Modal Dialog Sistemi ---
function modalGoster(baslik, mesaj, tip, butonlar) {
    var overlay = document.getElementById('modal-overlay');
    var ikonEl = document.getElementById('modal-ikon');
    var baslikEl = document.getElementById('modal-baslik');
    var mesajEl = document.getElementById('modal-mesaj');
    var btnAlani = document.getElementById('modal-butonlar');

    // Ikon
    var ikonlar = {
        hata: '\u26D4',
        uyari: '\u26A0\uFE0F',
        bilgi: '\u2139\uFE0F',
        soru: '\u2753',
        basari: '\u2705'
    };
    ikonEl.textContent = ikonlar[tip] || ikonlar.bilgi;
    baslikEl.textContent = baslik;
    mesajEl.textContent = mesaj;

    // Butonlar
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

// Sayfa yuklendikten sonra baslat
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('github-btn').addEventListener('click', githubAc);
    document.getElementById('profil-yenile-btn').addEventListener('click', function() {
        profilleriYukle({ sessiz: false });
    });
    document.getElementById('sifre-goster-btn').addEventListener('click', sifreGorunurlukDegistir);
    document.getElementById('kullanici').addEventListener('input', profilSeciminiTemizle);
    document.getElementById('sifre').addEventListener('input', profilSeciminiTemizle);
    baslatmaYukle();
});
