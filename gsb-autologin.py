import requests
import time
import urllib3

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# -------------------- Ayarlar --------------------
# ip adresi
IP_ADRESI = 'x.x.x.x' # GSB WiFi portal IP adresini buraya yazın

# login URL
LOGIN_URL = f'https://{IP_ADRESI}/j_spring_security_check'

# host name
HEADERS = {
    'Host': 'wifi.gsb.gov.tr'
}

# Giriş için gerekli form verileri
PAYLOAD = {
    'j_username': 'abcd',  # Kendi T.C. Kimlik Numaranızı yazın
    'j_password': 'abcd',  # Kendi GSB/KYK şifrenizi yazın
    'submit': 'Giriş',
}

# ---------------------------------------------------

def login():
    """Captive Portal'a giriş yapmayı dener."""
    print(f"[{time.strftime('%H:%M:%S')}]  Sunucuya giriş yapılıyor -> {IP_ADRESI}")
   
    try:
        response = requests.post(
            LOGIN_URL,
            data=PAYLOAD,
            headers=HEADERS,
            verify=False,  
            timeout=15    
        )

        # Başarılı giriş sonrası sayfa adresi "login" içermez
        if "login" not in response.url.lower() and "error" not in response.url.lower():
            print(f"[{time.strftime('%H:%M:%S')}] ✅ Giriş başarılı! İnternet bağlantısı aktif.")
        else:
            print(f"[{time.strftime('%H:%M:%S')}] ❌ Giriş başarısız. Kullanıcı adı veya şifrenizi kontrol edin.")

    except requests.exceptions.RequestException as e:
        print(f"[{time.strftime('%H:%M:%S')}] ❌ Bir ağ hatası oluştu.")
        print(f"Hata detayları: {e}")

if __name__ == '__main__':
    login()
