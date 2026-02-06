from PyQt6 import QtCore, QtGui, QtWidgets


class Ui_Form(object):
    def setupUi(self, Form):
        Form.setObjectName("Form")
        Form.resize(400, 300)
        Form.setStyleSheet("QWidget { background-color: transparent; }\n"
"QLabel { color: white; font-weight: bold; }\n"
"QLabel#baslikLabel { font-size: 20px; color: #2ecc71; }\n"
"QPushButton {\n"
"    background-color: #e74c3c; color: white; border-radius: 5px; padding: 8px; font-weight: bold;\n"
"}\n"
"QPushButton:hover { background-color: #c0392b; }")
        self.kapatBtn = QtWidgets.QPushButton(parent=Form)
        self.kapatBtn.setGeometry(QtCore.QRect(360, 0, 31, 31))
        self.kapatBtn.setStyleSheet("QPushButton {\n"
"    background-color: transparent;\n"
"    color: #7f8c8d;\n"
"    font-weight: bold;\n"
"    border: none;\n"
"    font-size: 14px;\n"
"}\n"
"QPushButton:hover {\n"
"    color: #c0392b;\n"
"}")
        self.kapatBtn.setObjectName("kapatBtn")
        self.altBtn = QtWidgets.QPushButton(parent=Form)
        self.altBtn.setGeometry(QtCore.QRect(320, 0, 41, 31))
        self.altBtn.setStyleSheet("QPushButton {\n"
"    background-color: transparent;\n"
"    color: #7f8c8d;\n"
"    font-weight: bold;\n"
"    border: none;\n"
"    font-size: 14px;\n"
"}\n"
"QPushButton:hover {\n"
"    color: #c0392b;\n"
"}")
        self.altBtn.setObjectName("altBtn")
        self.frame = QtWidgets.QFrame(parent=Form)
        self.frame.setGeometry(QtCore.QRect(-1, -1, 401, 301))
        self.frame.setStyleSheet("QFrame {\n"
"    background-color: #2c3e50;\n"
"    border-radius: 15px; \n"
"    border: 1px solid #34495e;\n"
"}")
        self.frame.setFrameShape(QtWidgets.QFrame.Shape.StyledPanel)
        self.frame.setFrameShadow(QtWidgets.QFrame.Shadow.Raised)
        self.frame.setObjectName("frame")
        self.adSoyadLabel = QtWidgets.QLabel(parent=self.frame)
        self.adSoyadLabel.setGeometry(QtCore.QRect(160, 50, 211, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.adSoyadLabel.setFont(font)
        self.adSoyadLabel.setObjectName("adSoyadLabel")
        self.kotaBilgisiLabel = QtWidgets.QLabel(parent=self.frame)
        self.kotaBilgisiLabel.setGeometry(QtCore.QRect(160, 100, 201, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.kotaBilgisiLabel.setFont(font)
        self.kotaBilgisiLabel.setObjectName("kotaBilgisiLabel")
        self.sonGirisBilgiLabel = QtWidgets.QLabel(parent=self.frame)
        self.sonGirisBilgiLabel.setGeometry(QtCore.QRect(160, 150, 191, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.sonGirisBilgiLabel.setFont(font)
        self.sonGirisBilgiLabel.setObjectName("sonGirisBilgiLabel")
        self.sonGirisLabel = QtWidgets.QLabel(parent=self.frame)
        self.sonGirisLabel.setGeometry(QtCore.QRect(60, 150, 71, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.sonGirisLabel.setFont(font)
        self.sonGirisLabel.setObjectName("sonGirisLabel")
        self.cotaLabel = QtWidgets.QLabel(parent=self.frame)
        self.cotaLabel.setGeometry(QtCore.QRect(50, 100, 91, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.cotaLabel.setFont(font)
        self.cotaLabel.setObjectName("cotaLabel")
        self.baslikLabel = QtWidgets.QLabel(parent=self.frame)
        self.baslikLabel.setGeometry(QtCore.QRect(10, 10, 171, 31))
        font = QtGui.QFont()
        font.setPointSize(-1)
        font.setBold(True)
        self.baslikLabel.setFont(font)
        self.baslikLabel.setObjectName("baslikLabel")
        self.cikisButton = QtWidgets.QPushButton(parent=self.frame)
        self.cikisButton.setGeometry(QtCore.QRect(310, 240, 81, 31))
        self.cikisButton.setObjectName("cikisButton")
        self.yenileBtn = QtWidgets.QPushButton(parent=self.frame)
        self.yenileBtn.setGeometry(QtCore.QRect(10, 100, 31, 31))
        font = QtGui.QFont()
        font.setPointSize(9)
        font.setBold(True)
        self.yenileBtn.setFont(font)
        self.yenileBtn.setAutoFillBackground(False)
        self.yenileBtn.setStyleSheet("QPushButton#yenileBtn {\n"
"    background-color: transparent;\n"
"    color: #f39c12;\n"
"    border: 2px solid #f39c12;\n"
"    border-radius: 15px;\n"
"    font-weight: bold;\n"
"}\n"
"\n"
"QPushButton#yenileBtn:hover {\n"
"    background-color: #f39c12;\n"
"    color: white;\n"
"}\n"
"\n"
"QPushButton#yenileBtn:pressed {\n"
"    background-color: #d35400;\n"
"    border: 2px solid #d35400;\n"
"}\n"
"\n"
"QPushButton#yenileBtn:disabled {\n"
"    border: 2px solid #7f8c8d;\n"
"    color: #7f8c8d;\n"
"    background-color: transparent;\n"
"}")
        self.yenileBtn.setObjectName("yenileBtn")
        self.welcomeLabel = QtWidgets.QLabel(parent=self.frame)
        self.welcomeLabel.setGeometry(QtCore.QRect(50, 50, 101, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.welcomeLabel.setFont(font)
        self.welcomeLabel.setObjectName("welcomeLabel")
        self.frame.raise_()
        self.kapatBtn.raise_()
        self.altBtn.raise_()

        self.retranslateUi(Form)
        QtCore.QMetaObject.connectSlotsByName(Form)

    def retranslateUi(self, Form):
        _translate = QtCore.QCoreApplication.translate
        Form.setWindowTitle(_translate("Form", "Form"))
        self.kapatBtn.setText(_translate("Form", "X"))
        self.altBtn.setText(_translate("Form", "_"))
        self.adSoyadLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">....</p></body></html>"))
        self.kotaBilgisiLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">....</p></body></html>"))
        self.sonGirisBilgiLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">.....</p></body></html>"))
        self.sonGirisLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">Son Giriş</p></body></html>"))
        self.cotaLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">Kalan Kota</p></body></html>"))
        self.baslikLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\"><span style=\" font-size:16pt;\">GSB AutoLogin</span></p></body></html>"))
        self.cikisButton.setText(_translate("Form", "Çıkış yap"))
        self.yenileBtn.setText(_translate("Form", "↻"))
        self.welcomeLabel.setText(_translate("Form", "<html><head/><body><p align=\"center\">Hoşgeldin</p></body></html>"))


if __name__ == "__main__":
    import sys
    app = QtWidgets.QApplication(sys.argv)
    Form = QtWidgets.QWidget()
    ui = Ui_Form()
    ui.setupUi(Form)
    Form.show()
    sys.exit(app.exec())
