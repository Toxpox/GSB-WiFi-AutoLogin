# -*- mode: python ; coding: utf-8 -*-
# GSB WiFi AutoLogin - PyInstaller Build Spec
# Build komutu: pyinstaller GSB_AutoLogin.spec

a = Analysis(
    ["src/gsb-autologin.py"],
    pathex=["src"],
    binaries=[],
    datas=[
        ("src/LoginPage.png", "."),
        ("src/wifi.png", "."),
        ("wifi.ico", "."),
    ],
    hiddenimports=[
        "customtkinter",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="GSB_AutoLogin",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon="wifi.ico",
)
