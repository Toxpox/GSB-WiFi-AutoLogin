const { invoke } = window.__TAURI__.core;

window.addEventListener('keydown', function(e) {
    if (e.key === 'F11') e.preventDefault();
}, true);
document.addEventListener('fullscreenchange', function() {
    if (document.fullscreenElement && document.exitFullscreen) {
        var p = document.exitFullscreen();
        if (p && p.catch) p.catch(function() {});
    }
});

let VERSION = "1.10.0";
let GIRIS_URL = "https://wifi.gsb.gov.tr/j_spring_security_check";
let KAYITLI_PROFILLER = [];
let SECILI_PROFIL_ID = null;
let VERSIYON_KONTROL_EDILDI = false;

const SVG = {
    close: '<svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12"/><path d="M18 6L6 18"/></svg>',
    eye: '<path d="M2.5 12s3.5-6.5 9.5-6.5S21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Z"/><circle cx="12" cy="12" r="2.6"/>',
    eyeOff: '<path d="M3 3l18 18"/><path d="M10.6 6.1A9.8 9.8 0 0 1 12 6c6 0 9.5 6 9.5 6a16.6 16.6 0 0 1-3.3 3.9"/><path d="M6.1 7.4A16.4 16.4 0 0 0 2.5 12s3.5 6 9.5 6a9.6 9.6 0 0 0 4.2-1"/><path d="M9.5 9.7a3 3 0 0 0 4.2 4.2"/>',
    pin: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M12 21s7-6.2 7-12a7 7 0 1 0-14 0c0 5.8 7 12 7 12Z"/><circle cx="12" cy="9.5" r="2.5"/></svg>',
    pencil: '<svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>',
    clock: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 7.5V12l3 2"/></svg>',
    modalIkon: {
        hata:    '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12"/><path d="M18 6L6 18"/></svg>',
        uyari:   '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3.5 22 20H2L12 3.5Z"/><path d="M12 10v5"/><circle cx="12" cy="17.5" r="0.8" fill="currentColor" stroke="none"/></svg>',
        bilgi:   '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 11v6"/><circle cx="12" cy="7.8" r="0.8" fill="currentColor" stroke="none"/></svg>',
        soru:    '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M9.2 9.3a3 3 0 1 1 4.4 2.6c-1.1.6-1.6 1.2-1.6 2.5"/><circle cx="12" cy="17" r="0.9" fill="currentColor" stroke="none"/></svg>',
        basari:  '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12.5l5 5L20 6.5"/></svg>',
    }
};

function ekranGoster(id) {
    document.querySelectorAll('.ekran').forEach(e => {
        e.classList.remove('aktif');
        e.classList.add('gizli');
    });
    const hedef = document.getElementById(id);
    if (!hedef) return;
    requestAnimationFrame(() => {
        hedef.classList.remove('gizli');
        hedef.classList.add('aktif');
    });
}

function durumGuncelle(metin, tip) {
    const cizgi = document.getElementById('durum');
    if (!cizgi) return;
    cizgi.className = 'statusline durum-' + (tip || 'bekle');
    const yazi = cizgi.querySelector('.durum-yazi');
    if (yazi) yazi.textContent = metin;
}

function yenidenBaglanmaDinle() {
    try {
        window.__TAURI__.event.listen('yeniden-baglanma', function(olay) {
            var durum = olay.payload || {};

            logYaz(durum.mesaj || 'Bağlantı kontrolü yapıldı', durum.tip || 'bilgi', true);
            if (durum.tip === 'basarili') durumGuncelle('Bağlı', 'basari');
            if (durum.tip === 'hata') durumGuncelle('Bağlantı koptu', 'hata');
        });
    } catch (_) {}
}

function tepsiOlaylariniDinle() {
    try {
        var ev = window.__TAURI__.event;
        ev.listen('tepsi-baglan', function() {
            if (document.getElementById('ekran-giris').classList.contains('aktif')) {
                girisBaslat(true);
            }
        });
        ev.listen('tepsi-cikis', function() {
            if (document.getElementById('ekran-hosgeldin').classList.contains('aktif')) {
                cikisYap();
            }
        });
    } catch (_) {}
}

async function agDurumunuKontrolEt() {
    try {
        var gsb = await invoke('gsb_aginda');
        if (!gsb) {
            logYaz('GSB ağında görünmüyorsunuz; bağlantı başarısız olabilir.', 'uyari');
            durumGuncelle('GSB ağı bulunamadı', 'bekle');
        }
        return gsb;
    } catch (_) {
        return true;
    }
}

async function baslatmaYukle() {
    const [appSonuc, ayarSonuc] = await Promise.all([
        invoke('app_bilgisi').catch(function () { return null; }),
        invoke('ayarlari_al').catch(function () { return null; }),
        profilleriYukle({ sessiz: true }),
    ]);

    if (appSonuc) {
        VERSION = appSonuc.version || VERSION;
        GIRIS_URL = appSonuc.giris_url || GIRIS_URL;
    }

    logYaz("GSB WiFi AutoLogin v" + VERSION, "bilgi");
    logYaz("Hazır.", "soluk");

    var gsbAginda = await agDurumunuKontrolEt();

    var ayarlar = ayarSonuc;
    if (ayarlar && ayarlar.otomatik_giris) {
        var kullanici = document.getElementById('kullanici').value;
        var sifre = document.getElementById('sifre').value;
        if (!kullanici || !sifre) {
            logYaz('Otomatik giriş: kayıtlı profil yok, atlandı.', 'soluk');
        } else if (!gsbAginda) {
            logYaz('Otomatik giriş: GSB ağı bulunamadı, atlandı.', 'uyari');
        } else {
            logYaz('Otomatik giriş başlatılıyor…', 'bilgi');
            girisBaslat(true);
        }
    }
}

async function ayarlariAc() {
    try {
        var a = await invoke('ayarlari_al');
        document.getElementById('ayar-otomatik-giris').checked = !!a.otomatik_giris;
        document.getElementById('ayar-tepsiye-kucul').checked = !!a.tepsiye_kucul;
        document.getElementById('ayar-baslangic').checked = !!a.baslangicta_calis;
        document.getElementById('ayar-yeniden-baglan').checked = !!a.yeniden_baglan;
        document.getElementById('ayar-kota-bildirim').checked = !!a.kota_bildirim;
        document.getElementById('ayar-overlay').classList.remove('gizli');
    } catch (e) {
        await modalUyari('Ayarlar Açılamadı', String(e));
    }
}

async function ayarlariKaydet() {
    var ayarlar = {
        otomatik_giris: document.getElementById('ayar-otomatik-giris').checked,
        tepsiye_kucul: document.getElementById('ayar-tepsiye-kucul').checked,
        baslangicta_calis: document.getElementById('ayar-baslangic').checked,
        yeniden_baglan: document.getElementById('ayar-yeniden-baglan').checked,
        kota_bildirim: document.getElementById('ayar-kota-bildirim').checked,
    };
    try {
        await invoke('ayarlari_kaydet', { ayarlar: ayarlar });
        document.getElementById('ayar-overlay').classList.add('gizli');
        logYaz('Ayarlar kaydedildi.', 'soluk');
    } catch (e) {
        await modalUyari('Ayarlar Kaydedilemedi', String(e));
    }
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
    if (m) return m[0].slice(0, 2);

    var harfler = String(label).match(/[\p{L}\d]/gu);
    return (harfler ? harfler.slice(0, 2).join('') : '??').toUpperCase() || '??';
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
        const etiket = profil.takma_ad || profil.masked_username || 'Profil';
        const token = document.createElement('div');
        token.className = 'token' + (aktif ? ' aktif' : '');

        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'token-btn';

        btn.title = profil.takma_ad
            ? profil.takma_ad + ' (' + (profil.masked_username || '?') + ')'
            : (profil.masked_username || 'Profil');
        btn.addEventListener('click', function() { profilSec(profil.id); });

        const avatar = document.createElement('span');
        avatar.className = 'token-avatar';
        avatar.textContent = profilBasHarfleri(etiket);
        btn.appendChild(avatar);

        if (aktif) {
            const name = document.createElement('span');
            name.className = 'token-name';
            name.textContent = etiket;
            btn.appendChild(name);
        }

        const duzenle = document.createElement('button');
        duzenle.type = 'button';
        duzenle.className = 'token-duzenle';
        duzenle.title = 'Profili adlandır';
        duzenle.setAttribute('aria-label', 'Profili adlandır');
        duzenle.innerHTML = SVG.pencil;
        duzenle.addEventListener('click', function(e) {
            e.stopPropagation();
            takmaAdDiyalogAc(profil);
        });

        const del = document.createElement('button');
        del.type = 'button';
        del.className = 'token-del';
        del.title = 'Profili sil';
        del.setAttribute('aria-label', 'Profili sil');
        del.innerHTML = SVG.close;
        del.addEventListener('click', function(e) {
            e.stopPropagation();
            profilSil(profil.id, etiket);
        });

        token.appendChild(btn);
        token.appendChild(duzenle);
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

            const ad = (profil && (profil.takma_ad || profil.masked_username)) || 'profil';
            logYaz('Profil seçildi: ' + ad, 'soluk');
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

function takmaAdDiyalogAc(profil) {
    var overlay = document.getElementById('takma-ad-overlay');
    var input = document.getElementById('takma-ad-input');
    input.value = profil.takma_ad || '';
    overlay.dataset.profilId = profil.id;
    overlay.classList.remove('gizli');
    input.focus();
    input.select();
}

function takmaAdDiyalogKapat() {
    var overlay = document.getElementById('takma-ad-overlay');
    overlay.dataset.profilId = '';
    overlay.classList.add('gizli');
}

async function takmaAdKaydet() {
    var overlay = document.getElementById('takma-ad-overlay');
    var id = overlay.dataset.profilId;
    if (!id) {
        takmaAdDiyalogKapat();
        return;
    }
    var ad = document.getElementById('takma-ad-input').value;
    try {
        KAYITLI_PROFILLER = await invoke('profil_takma_ad_ayarla', { id: id, takmaAd: ad });
        profilListesiniCiz();
        takmaAdDiyalogKapat();
        logYaz(ad.trim() ? 'Profil adlandırıldı: ' + ad.trim() : 'Takma ad kaldırıldı.', 'soluk');
    } catch (e) {
        await modalUyari('Adlandırılamadı', String(e));
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
        const bilgi = await invoke('guncelleme_kontrol');
        if (bilgi && bilgi.surum) {
            logYaz('Yeni sürüm mevcut: v' + bilgi.surum, 'uyari');
            guncellemeBariGoster('Yeni sürüm v' + bilgi.surum + ' — Güncellemek için tıkla', guncellemeKur);
        }
        return;
    } catch (_) {
    }

    try {
        const sonuc = await invoke('yeni_versiyon_kontrol');
        if (sonuc.guncel || !sonuc.release_url) return;

        const son = sonuc.son || 'yeni sürüm';
        logYaz('Yeni sürüm mevcut: ' + son, 'uyari');
        guncellemeBariGoster('Yeni sürüm ' + son + ' — İndirme sayfasını aç', function() {
            invoke('github_link_ac', { url: sonuc.release_url }).catch(function(e) {
                logYaz('Release sayfası açılamadı: ' + String(e), 'uyari');
            });
        });
    } catch (e) {
        logYaz('Sürüm kontrolü yapılamadı: ' + String(e), 'uyari');
    }
}

function guncellemeBariGoster(metin, tiklama) {
    var btn = document.getElementById('guncelleme-btn');
    var yazi = document.getElementById('guncelleme-btn-yazi');
    if (!btn || !yazi) return;
    btn.title = metin;
    yazi.textContent = 'Güncelle';
    btn.onclick = tiklama;
    btn.disabled = false;
    btn.classList.remove('gizli');
}

async function guncellemeKur() {
    var btn = document.getElementById('guncelleme-btn');
    var yazi = document.getElementById('guncelleme-btn-yazi');
    if (!btn || !yazi) return;
    btn.disabled = true;
    yazi.textContent = '…';
    btn.title = 'İndiriliyor…';

    var dinlemeyiBirak = null;
    try {
        dinlemeyiBirak = await window.__TAURI__.event.listen('guncelleme-ilerleme', function(olay) {
            var p = olay.payload || {};
            if (p.yuzde !== null && p.yuzde !== undefined) {
                yazi.textContent = '%' + p.yuzde;
                btn.title = 'İndiriliyor… %' + p.yuzde;
            } else {
                yazi.textContent = (p.indirilen_mb || 0).toFixed(1) + ' MB';
            }
        });
        logYaz('Güncelleme indiriliyor.', 'bilgi');
        await invoke('guncelleme_kur');
        yazi.textContent = 'Kuruluyor…';
        btn.title = 'Kuruluyor, uygulama yeniden başlayacak…';
    } catch (e) {
        logYaz('Güncelleme başarısız: ' + String(e), 'hata');
        yazi.textContent = 'Tekrar dene';
        btn.title = 'Güncelleme başarısız — tekrar denemek için tıkla';
        btn.disabled = false;

        btn.onclick = function() {
            VERSIYON_KONTROL_EDILDI = false;
            btn.classList.add('gizli');
            yeniVersiyonKontrolEt();
        };
    } finally {
        if (dinlemeyiBirak) dinlemeyiBirak();
    }
}

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
    document.getElementById('ayar-btn').addEventListener('click', ayarlariAc);
    document.getElementById('ayar-kaydet').addEventListener('click', ayarlariKaydet);
    document.getElementById('ayar-iptal').addEventListener('click', function() {
        document.getElementById('ayar-overlay').classList.add('gizli');
    });
    document.getElementById('takma-ad-kaydet').addEventListener('click', takmaAdKaydet);
    document.getElementById('takma-ad-iptal').addEventListener('click', takmaAdDiyalogKapat);
    document.getElementById('takma-ad-input').addEventListener('keydown', function(e) {
        if (e.key === 'Enter') takmaAdKaydet();
        if (e.key === 'Escape') takmaAdDiyalogKapat();
    });
    yenidenBaglanmaDinle();
    tepsiOlaylariniDinle();
    baslatmaYukle();
});
