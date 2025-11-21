from PyQt6 import QtCore, QtGui, QtWidgets


class Ui_LoginPage(object):
    def setupUi(self, LoginPage):
        LoginPage.setObjectName("LoginPage")
        LoginPage.resize(800, 600)
        LoginPage.setStyleSheet("")
        self.centralwidget = QtWidgets.QWidget(parent=LoginPage)
        self.centralwidget.setObjectName("centralwidget")
        self.frame = QtWidgets.QFrame(parent=self.centralwidget)
        self.frame.setGeometry(QtCore.QRect(0, 0, 341, 231))
        self.frame.setStyleSheet("/* Ana Arka Plan */\n"
"#centralwidget {\n"
"    background-color: #2c3e50;\n"
"}\n"
"\n"
"#frame {\n"
"    background-color: #ffffff;\n"
"    border-radius: 15px;\n"
"    border: 1px solid #dcdcdc;\n"
"}\n"
"\n"
"/* Yazı Kutuları (Username & Password) */\n"
"QLineEdit {\n"
"    border: 1px solid #bdc3c7;\n"
"    border-radius: 5px;\n"
"    padding: 5px;\n"
"    background-color: #ecf0f1;\n"
"}\n"
"/* Yazı Kutuları (Username & Password) */\n"
"QLineEdit {\n"
"    border: 1px solid #bdc3c7;\n"
"    border-radius: 5px;\n"
"    padding: 5px;\n"
"    background-color: #ecf0f1;\n"
"    color: #2c3e50;\n"
"    font-weight: 500;\n"
"}\n"
"\n"
"QLineEdit:focus {\n"
"    border: 2px solid #3498db;\n"
"}\n"
"\n"
"/* Giriş Yap Butonu */\n"
"QPushButton {\n"
"    background-color: #2980b9;\n"
"    color: white;\n"
"    border-radius: 5px;\n"
"    font-weight: bold;\n"
"}\n"
"QPushButton:hover {\n"
"    background-color: #3498db;\n"
"}\n"
"QPushButton:pressed {\n"
"    background-color: #1f618d;\n"
"}\n"
"\n"
"/* Etiketler */\n"
"QLabel {\n"
"    color: #333333;\n"
"    font-weight: 500;\n"
"}")
        self.frame.setFrameShape(QtWidgets.QFrame.Shape.StyledPanel)
        self.frame.setFrameShadow(QtWidgets.QFrame.Shadow.Raised)
        self.frame.setObjectName("frame")
        self.label = QtWidgets.QLabel(parent=self.frame)
        self.label.setGeometry(QtCore.QRect(10, 10, 241, 21))
        font = QtGui.QFont()
        font.setPointSize(12)
        self.label.setFont(font)
        self.label.setObjectName("label")
        self.label_2 = QtWidgets.QLabel(parent=self.frame)
        self.label_2.setGeometry(QtCore.QRect(30, 40, 91, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        self.label_2.setFont(font)
        self.label_2.setObjectName("label_2")
        self.label_3 = QtWidgets.QLabel(parent=self.frame)
        self.label_3.setGeometry(QtCore.QRect(30, 90, 101, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        self.label_3.setFont(font)
        self.label_3.setObjectName("label_3")
        self.BeniHatirla = QtWidgets.QCheckBox(parent=self.frame)
        self.BeniHatirla.setGeometry(QtCore.QRect(200, 130, 101, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        self.BeniHatirla.setFont(font)
        self.BeniHatirla.setAutoFillBackground(False)
        self.BeniHatirla.setStyleSheet("/* Checkbox Genel Ayarları */\n"
"QCheckBox {\n"
"    spacing: 8px;\n"
"    color: #555555;\n"
"    font-weight: 500;\n"
"    background-color: transparent;\n"
"}\n"
"\n"
"QCheckBox::indicator {\n"
"    width: 16px;\n"
"    height: 16px;\n"
"    border: 1px solid #bdc3c7;\n"
"    border-radius: 3px;\n"
"    background-color: white;\n"
"}\n"
"\n"
"/* Fare üzerine gelince kutu rengi */\n"
"QCheckBox::indicator:hover {\n"
"    border: 1px solid #3498db;\n"
"}\n"
"\n"
"QCheckBox::indicator:checked {\n"
"    background-color: #3498db;\n"
"    border: 1px solid #3498db;\n"
"}")
        self.BeniHatirla.setObjectName("BeniHatirla")
        self.KullaniciAdi = QtWidgets.QLineEdit(parent=self.frame)
        self.KullaniciAdi.setGeometry(QtCore.QRect(180, 40, 121, 31))
        self.KullaniciAdi.setObjectName("KullaniciAdi")
        self.Sifre = QtWidgets.QLineEdit(parent=self.frame)
        self.Sifre.setGeometry(QtCore.QRect(180, 90, 121, 31))
        self.Sifre.setEchoMode(QtWidgets.QLineEdit.EchoMode.Password)
        self.Sifre.setObjectName("Sifre")
        self.GirisButton = QtWidgets.QPushButton(parent=self.frame)
        self.GirisButton.setGeometry(QtCore.QRect(130, 180, 81, 31))
        self.GirisButton.setObjectName("GirisButton")
        LoginPage.setCentralWidget(self.centralwidget)
        self.menubar = QtWidgets.QMenuBar(parent=LoginPage)
        self.menubar.setGeometry(QtCore.QRect(0, 0, 800, 17))
        self.menubar.setObjectName("menubar")
        LoginPage.setMenuBar(self.menubar)
        self.statusbar = QtWidgets.QStatusBar(parent=LoginPage)
        self.statusbar.setObjectName("statusbar")
        LoginPage.setStatusBar(self.statusbar)

        self.retranslateUi(LoginPage)
        QtCore.QMetaObject.connectSlotsByName(LoginPage)
        LoginPage.setTabOrder(self.KullaniciAdi, self.Sifre)
        LoginPage.setTabOrder(self.Sifre, self.BeniHatirla)
        LoginPage.setTabOrder(self.BeniHatirla, self.GirisButton)

    def retranslateUi(self, LoginPage):
        _translate = QtCore.QCoreApplication.translate
        LoginPage.setWindowTitle(_translate("LoginPage", "LoginPage"))
        self.label.setText(_translate("LoginPage", "GSB Otomatik Giriş"))
        self.label_2.setText(_translate("LoginPage", "Kullanıcı Adı:"))
        self.label_3.setText(_translate("LoginPage", "Şifre:"))
        self.BeniHatirla.setText(_translate("LoginPage", "Beni Hatırla"))
        self.GirisButton.setText(_translate("LoginPage", "Giriş Yap"))


if __name__ == "__main__":
    import sys
    app = QtWidgets.QApplication(sys.argv)
    LoginPage = QtWidgets.QMainWindow()
    ui = Ui_LoginPage()
    ui.setupUi(LoginPage)
    LoginPage.show()
    sys.exit(app.exec())
