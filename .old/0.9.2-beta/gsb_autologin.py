import sys
import json
import requests
import urllib3
import datetime
import re
from pathlib import Path
from bs4 import BeautifulSoup

from PyQt6.QtWidgets import QApplication, QMainWindow, QMessageBox, QWidget, QStackedWidget
from PyQt6.QtCore import Qt, QPoint, QThread, pyqtSignal

from login_ui import Ui_LoginPage
from dashboard_ui import Ui_Form as Ui_Dashboard 

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

LOGIN_URL = "https://wifi.gsb.gov.tr/j_spring_security_check"
LOGOUT_URL = "https://wifi.gsb.gov.tr/cikisSon.html?logout=1"
USER_CONFIG_PATH = Path(__file__).with_name("user_config.json")

def extract_portal_info(html):
    """HTML'den İsim, Son Giriş ve Kotayı cımbızla çeker."""
    soup = BeautifulSoup(html, "html.parser")
    
    data = {
        "isim": "Öğrenci",
        "kota": "Bilinmiyor",
        "zaman": datetime.datetime.now().strftime("%d.%m.%Y %H:%M")
    }

    try:
        name_span = soup.select_one("span.myinfo")
        if name_span:
            raw_name = name_span.get_text(strip=True)
            data["isim"] = raw_name.title()
    except: pass

    try:
        login_label = soup.find("label", class_="myinfo", string=re.compile("Son Giriş", re.IGNORECASE))
        if login_label:
            full_text = login_label.get_text(strip=True)
            if ":" in full_text:
                data["zaman"] = full_text.split(":", 1)[1].strip()
    except: pass

    try:
        quota_label = soup.find("label", id=re.compile(r"mainPanel:kota:j_idt101:0:j_idt123"))
        if quota_label:
            raw_quota = quota_label.get_text(strip=True)
            if "." in raw_quota:
                raw_quota = raw_quota.split(".")[0]
            data["kota"] = f"{raw_quota} MB"
    except: pass

    return data

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
    success = pyqtSignal(dict)
    error = pyqtSignal(str)

    def __init__(self, session, user, pwd):
        super().__init__()
        self.session = session
        self.user = user
        self.pwd = pwd

    def run(self):
        try:
            payload = {"j_username": self.user, "j_password": self.pwd, "submit": "Giriş"}
            response = self.session.post(LOGIN_URL, data=payload, verify=False, timeout=15)
            response.raise_for_status()
            data_dict = extract_portal_info(response.text)
            self.success.emit(data_dict)
        except Exception as e:
            self.error.emit(str(e))
        finally:
            self.finished.emit()

class RefreshWorker(QThread):
    finished = pyqtSignal()
    success = pyqtSignal(dict)
    error = pyqtSignal(str)

    def __init__(self, session):
        super().__init__()
        self.session = session

    def run(self):
        try:
            response = self.session.get("https://wifi.gsb.gov.tr/", verify=False, timeout=10)
            data_dict = extract_portal_info(response.text)
            self.success.emit(data_dict)
        except Exception as e:
            self.error.emit(str(e))
        finally:
            self.finished.emit()

class LogoutWorker(QThread):
    finished = pyqtSignal()
    def __init__(self, session):
        super().__init__()
        self.session = session

    def run(self):
        try:
            self.session.get(LOGOUT_URL, verify=False, timeout=5)
        except: pass
        self.finished.emit()


class MainApp(QMainWindow):
    def __init__(self):
        super().__init__()
        
        self.session = requests.Session()

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

        if hasattr(self.ui_dash, 'pushButton'): self.ui_dash.pushButton.clicked.connect(self.logout)
        elif hasattr(self.ui_dash, 'cikisButton'): self.ui_dash.cikisButton.clicked.connect(self.logout)
        
        if hasattr(self.ui_dash, 'yenileBtn'):
            self.ui_dash.yenileBtn.clicked.connect(self.start_refresh)
            
            yenile_style = """
            QPushButton {
                background-color: #16a085;
                color: white;
                border-radius: 8px;
                border: none;
                font-weight: bold;
                font-size: 13px;
                padding: 6px 12px;
            }
            QPushButton:hover {
                background-color: #1abc9c;
                border: 1px solid #a3e4d7;
            }
            QPushButton:pressed {
                background-color: #0e6655;
                padding-top: 8px;
                padding-left: 8px;
            }
            QPushButton:disabled {
                background-color: #7f8c8d;
                color: #bdc3c7;
            }
            """
            self.ui_dash.yenileBtn.setStyleSheet(yenile_style)
        
        if hasattr(self.ui_dash, 'kapatBtn'): self.ui_dash.kapatBtn.clicked.connect(self.close)
        if hasattr(self.ui_dash, 'altBtn'): self.ui_dash.altBtn.clicked.connect(self.showMinimized)

        self.stack.addWidget(self.dash_widget)


    def start_login(self):
        user = self.ui_login.KullaniciAdi.text().strip()
        pwd = self.ui_login.Sifre.text()

        if not user or not pwd: 
            QMessageBox.warning(self, "Uyarı", "Bilgileri giriniz.")
            return

        if self.ui_login.BeniHatirla.isChecked(): save_user(user)

        self.ui_login.GirisButton.setEnabled(False)
        self.ui_login.GirisButton.setText("Bağlanıyor...")

        self.worker = LoginWorker(self.session, user, pwd)
        self.worker.success.connect(self.on_success_update)
        self.worker.error.connect(self.on_login_error)
        self.worker.finished.connect(lambda: self.ui_login.GirisButton.setEnabled(True))
        self.worker.finished.connect(lambda: self.ui_login.GirisButton.setText("Giriş Yap"))
        self.worker.start()

    def start_refresh(self):
        if hasattr(self.ui_dash, 'yenileBtn'):
            self.ui_dash.yenileBtn.setEnabled(False)
            self.ui_dash.yenileBtn.setText("...")
        

    def on_refresh_finished(self):
        if hasattr(self.ui_dash, 'yenileBtn'):
            self.ui_dash.yenileBtn.setEnabled(True)
            self.ui_dash.yenileBtn.setText("Yenile")

    def on_success_update(self, data):
        self.ui_dash.adSoyadLabel.setText(data.get("isim", "Öğrenci"))
        self.ui_dash.kotaBilgisiLabel.setText(data.get("kota", "Bilinmiyor"))
        self.ui_dash.sonGirisBilgiLabel.setText(data.get("zaman", ""))

        self.stack.setCurrentWidget(self.dash_widget)
        if hasattr(self.ui_login, 'statusbar'): self.ui_login.statusbar.showMessage("Online")

    def on_login_error(self, err_msg):
        QMessageBox.warning(self, "Hata", f"İşlem Başarısız:\n{err_msg}")

    def logout(self):
        btn = getattr(self.ui_dash, 'pushButton', None) or getattr(self.ui_dash, 'cikisButton', None)
        if btn:
            btn.setEnabled(False)
            btn.setText("Çıkış...")
            
        self.logout_worker = LogoutWorker(self.session)
        self.logout_worker.finished.connect(self.on_logout_finished)
        self.logout_worker.start()

    def on_logout_finished(self):
        self.stack.setCurrentWidget(self.login_widget)
        self.ui_login.Sifre.clear()
        
        self.session = requests.Session()
        
        btn = getattr(self.ui_dash, 'pushButton', None) or getattr(self.ui_dash, 'cikisButton', None)
        if btn:
            btn.setEnabled(True)
            btn.setText("Çıkış Yap") 

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton: self.oldPos = event.globalPosition().toPoint()
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