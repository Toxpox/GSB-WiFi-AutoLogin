import sys
import json
import socket
import requests
import urllib3
import datetime
from pathlib import Path
from urllib.parse import urlparse
from bs4 import BeautifulSoup

from PyQt6.QtWidgets import QApplication, QMainWindow, QMessageBox, QWidget, QStackedWidget
from PyQt6.QtCore import Qt, QPoint, QThread, pyqtSignal

from login_ui import Ui_LoginPage
from dashboard_ui import Ui_Form as Ui_Dashboard 

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)
DEFAULT_LOGIN_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
LOGOUT_URL = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1"
USER_CONFIG_PATH = Path(__file__).with_name("user_config.json")

TEST_MODE = False

def perform_login(login_url, username, password):
    session = requests.Session()
    payload = {"j_username": username, "j_password": password, "submit": "Giriş"}
    try:
        response = session.post(login_url, data=payload, verify=False, timeout=10)
        response.raise_for_status()
        return response.text
    except Exception as e:
        raise e

def extract_portal_info(html):
    if "çıkış" in html.lower() or "logout" in html.lower():
        soup = BeautifulSoup(html, "html.parser")
        center = soup.select_one("#content-div > center")
        if center:
            return center.get_text(separator=" ", strip=True)
        return "Giriş Başarılı (Kota bilgisi okunamadı)"
    return "Bilgi alınamadı."

def load_user():
    if USER_CONFIG_PATH.exists():
        try: return json.loads(USER_CONFIG_PATH.read_text(encoding="utf-8")).get("username", "")
        except: return ""
    return ""

def save_user(username):
    try: USER_CONFIG_PATH.write_text(json.dumps({"username": username}), encoding="utf-8")
    except: pass


class LoginWorker(QThread):
    finished = pyqtSignal()
    success = pyqtSignal(str)
    error = pyqtSignal(str)

    def __init__(self, user, pwd):
        super().__init__()
        self.user = user
        self.pwd = pwd
    
    def run(self):
        try:
            result_html = perform_login(DEFAULT_LOGIN_URL, self.user, self.pwd)
            info = extract_portal_info(result_html)
            
            if "Bilgi alınamadı" not in info:
                self.success.emit(info)
            else:
                self.error.emit("Giriş yapıldı fakat portal bilgisi alınamadı veya hatalı giriş.")
        except Exception as e:
            self.error.emit(str(e))
        finally:
            self.finished.emit()

class LogoutWorker(QThread):
    finished = pyqtSignal()

    def run(self):
        try:
            requests.get(LOGOUT_URL, verify=False, timeout=5)
        except Exception:
            pass
        self.finished.emit()


class MainApp(QMainWindow):
    def __init__(self):
        super().__init__()
        
        self.setWindowFlags(Qt.WindowType.FramelessWindowHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.resize(450, 350)
        self.stack = QStackedWidget()
        self.setCentralWidget(self.stack)
        self.setup_login_page()
        self.setup_dashboard_page()
        self.stack.setCurrentWidget(self.login_widget)

    def setup_login_page(self):
        self.login_widget = QMainWindow() 
        self.ui_login = Ui_LoginPage()
        self.ui_login.setupUi(self.login_widget)
        
        self.ui_login.GirisButton.clicked.connect(self.start_login)
        self.ui_login.Sifre.returnPressed.connect(self.start_login)
        
        if hasattr(self.ui_login, 'kapatBtn'): self.ui_login.kapatBtn.clicked.connect(self.close)
        if hasattr(self.ui_login, 'altBtn'): self.ui_login.altBtn.clicked.connect(self.showMinimized)
        
        saved_user = load_user()
        if saved_user:
            self.ui_login.KullaniciAdi.setText(saved_user)
            self.ui_login.BeniHatirla.setChecked(True)

        self.stack.addWidget(self.login_widget)

    def setup_dashboard_page(self):
        self.dash_widget = QWidget()
        self.ui_dash = Ui_Dashboard()
        self.ui_dash.setupUi(self.dash_widget)

        if hasattr(self.ui_dash, 'cikisButton'):
            self.ui_dash.cikisButton.clicked.connect(self.logout)
        
        if hasattr(self.ui_dash, 'kapatBtn'): self.ui_dash.kapatBtn.clicked.connect(self.close)
        if hasattr(self.ui_dash, 'altBtn'): self.ui_dash.altBtn.clicked.connect(self.showMinimized)

        self.stack.addWidget(self.dash_widget)

    def start_login(self):
        user = self.ui_login.KullaniciAdi.text().strip()
        pwd = self.ui_login.Sifre.text()

        if not user or not pwd: 
            QMessageBox.warning(self, "Uyarı", "Kullanıcı adı ve şifre boş olamaz.")
            return

        if self.ui_login.BeniHatirla.isChecked(): save_user(user)

        self.ui_login.GirisButton.setEnabled(False)
        self.ui_login.GirisButton.setText("Bağlanıyor...")
        if hasattr(self.ui_login, 'statusbar'):
            self.ui_login.statusbar.showMessage("Sunucuya istek gönderiliyor...")

        self.worker = LoginWorker(user, pwd)
        self.worker.success.connect(self.on_login_success)
        self.worker.error.connect(self.on_login_error)
        self.worker.finished.connect(lambda: self.ui_login.GirisButton.setEnabled(True))
        self.worker.finished.connect(lambda: self.ui_login.GirisButton.setText("Giriş Yap"))
        self.worker.start()

    def on_login_success(self, info_text):
        user = self.ui_login.KullaniciAdi.text()
        if hasattr(self.ui_dash, 'adSoyadLabel'):
            self.ui_dash.adSoyadLabel.setText(user)
        
        if len(info_text) > 50: info_text = info_text[:50] + "..."
        if hasattr(self.ui_dash, 'kotaBilgisiLabel'):
            self.ui_dash.kotaBilgisiLabel.setText(info_text)
        
        now = datetime.datetime.now().strftime("%d.%m.%Y %H:%M")
        if hasattr(self.ui_dash, 'sonGirisBilgiLabel'):
            self.ui_dash.sonGirisBilgiLabel.setText(now)

        self.stack.setCurrentWidget(self.dash_widget)
        if hasattr(self.ui_login, 'statusbar'):
            self.ui_login.statusbar.showMessage("Online")

    def on_login_error(self, err_msg):
        QMessageBox.warning(self, "Giriş Başarısız", f"Hata Detayı:\n{err_msg}")
        if hasattr(self.ui_login, 'statusbar'):
            self.ui_login.statusbar.showMessage("Giriş başarısız.")

    def logout(self):
        if hasattr(self.ui_dash, 'cikisButton'):
            self.ui_dash.cikisButton.setEnabled(False)
            self.ui_dash.cikisButton.setText("Çıkış Yapılıyor...")
            
        self.logout_worker = LogoutWorker()
        self.logout_worker.finished.connect(self.on_logout_finished)
        self.logout_worker.start()

    def on_logout_finished(self):
        self.stack.setCurrentWidget(self.login_widget)
        self.ui_login.Sifre.clear()
        
        if hasattr(self.ui_dash, 'cikisButton'):
            self.ui_dash.cikisButton.setEnabled(True)
            self.ui_dash.cikisButton.setText("Çıkış Yap") 

        if hasattr(self.ui_login, 'statusbar'):
            self.ui_login.statusbar.showMessage("Başarıyla çıkış yapıldı.")

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton: 
            self.oldPos = event.globalPosition().toPoint()
            
    def mouseMoveEvent(self, event):
        if hasattr(self, 'oldPos'):
            delta = QPoint(event.globalPosition().toPoint() - self.oldPos)
            self.move(self.x() + delta.x(), self.y() + delta.y())
            self.oldPos = event.globalPosition().toPoint()

if __name__ == "__main__":
    app = QApplication(sys.argv)
    win = MainApp()
    win.show()
    sys.exit(app.exec())