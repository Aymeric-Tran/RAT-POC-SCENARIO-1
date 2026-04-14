# Remote Access Trojan (RAT)

## Auteurs
- Ryan GUESMIA — 3SI2
- Aymeric TRAN — 3SI2

## Description

Ce projet est une implémentation d’un Remote Access Trojan (RAT), un programme de prise de contrôle à distance, développé à des fins éducatives. Il permet notamment :

-  Enregistrement des frappes clavier (keylogger)
-  Capture d’écran
-  Enregistrement du micro
-  Analyse du trafic réseau
-  Récupération de journaux système
-  Furtivité et persistance dans le système
-  Connexion à un serveur de commande et de contrôle (C2)
-  Kill switch (arrêt du malware depuis le C2)
-  Suppression automatique du malware depuis le C2


L’objectif principal est de **comprendre le fonctionnement des malwares**, en particulier leurs capacités de dissimulation et de persistance, dans un cadre strictement pédagogique et contrôlé.

---

## Compilation

### Prérequis

- [Rust](https://www.rust-lang.org/)
- Librairies spécifiques :
  - `x11` (Linux)
  - `winapi` (Windows)

### Compilation

```bash
cargo build
```

### Créer l'executable 

windows 

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
```

Linux
```bash
dpkg-deb --build netflix_1.0.0_amd64
```

### Lancement en local 

```bash
cargo r
```