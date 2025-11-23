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
        self.welcomeLabel = QtWidgets.QLabel(parent=Form)
        self.welcomeLabel.setGeometry(QtCore.QRect(40, 50, 81, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.welcomeLabel.setFont(font)
        self.welcomeLabel.setObjectName("welcomeLabel")
        self.cikisButton = QtWidgets.QPushButton(parent=Form)
        self.cikisButton.setGeometry(QtCore.QRect(300, 260, 81, 31))
        self.cikisButton.setObjectName("cikisButton")
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
        self.kotaBilgisiLabel.setGeometry(QtCore.QRect(160, 90, 211, 41))
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
        self.sonGirisLabel.setGeometry(QtCore.QRect(40, 150, 81, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.sonGirisLabel.setFont(font)
        self.sonGirisLabel.setObjectName("sonGirisLabel")
        self.cotaLabel = QtWidgets.QLabel(parent=self.frame)
        self.cotaLabel.setGeometry(QtCore.QRect(40, 100, 111, 31))
        font = QtGui.QFont()
        font.setPointSize(11)
        font.setBold(True)
        self.cotaLabel.setFont(font)
        self.cotaLabel.setObjectName("cotaLabel")
        self.frame.raise_()
        self.kapatBtn.raise_()
        self.altBtn.raise_()
        self.welcomeLabel.raise_()
        self.cikisButton.raise_()

        self.retranslateUi(Form)
        QtCore.QMetaObject.connectSlotsByName(Form)

    def retranslateUi(self, Form):
        _translate = QtCore.QCoreApplication.translate
        Form.setWindowTitle(_translate("Form", "Form"))
        self.kapatBtn.setText(_translate("Form", "X"))
        self.altBtn.setText(_translate("Form", "_"))
        self.welcomeLabel.setText(_translate("Form", "Hoşgeldin"))
        self.cikisButton.setText(_translate("Form", "Çıkış yap"))
        self.adSoyadLabel.setText(_translate("Form", "...."))
        self.kotaBilgisiLabel.setText(_translate("Form", "...."))
        self.sonGirisBilgiLabel.setText(_translate("Form", "....."))
        self.sonGirisLabel.setText(_translate("Form", "Son Giriş"))
        self.cotaLabel.setText(_translate("Form", "Kalan Kota"))


if __name__ == "__main__":
    import sys
    app = QtWidgets.QApplication(sys.argv)
    Form = QtWidgets.QWidget()
    ui = Ui_Form()
    ui.setupUi(Form)
    Form.show()
    sys.exit(app.exec())
