# 0-shell

Un shell Unix minimaliste écrit en **Rust**, sans aucune dépendance externe
(uniquement la bibliothèque standard). Inspiré de [BusyBox](https://busybox.net/),
il implémente les commandes Unix essentielles « à la main » via les appels
système, sans jamais lancer de binaire externe (`bash`, `ls`, `cat`, …).

## Fonctionnalités

- Boucle interactive avec invite (*prompt*) affichant le répertoire courant
- Analyse des commandes avec gestion des guillemets, des échappements et des
  variables d'environnement
- Sortie propre via `exit` ou `Ctrl+D` (EOF)
- Commandes intégrées implémentées depuis zéro
- Plusieurs bonus (voir plus bas)

## Compilation et exécution

Prérequis : [Rust](https://www.rust-lang.org/tools/install) (édition 2021).

```sh
# Compiler
cargo build --release

# Lancer
cargo run            # ou directement :
./target/release/0-shell
```

Exemple de session :

```text
~/projects/0-shell $ echo "Hello There"
Hello There
~/projects/0-shell $ pwd
/home/user/projects/0-shell
~/projects/0-shell $ ls -F
src/  Cargo.toml  README.md
~/projects/0-shell $ exit
```

## Commandes disponibles

| Commande | Options | Description |
|----------|---------|-------------|
| `echo`   | `-n`    | Affiche ses arguments (`-n` supprime le saut de ligne final) |
| `cd`     |         | Change de répertoire ; sans argument va vers `$HOME` |
| `pwd`    |         | Affiche le répertoire de travail courant |
| `ls`     | `-l` `-a` `-F` | Liste le contenu d'un répertoire |
| `cat`    |         | Affiche le contenu de fichiers (ou de l'entrée standard) |
| `cp`     | `-r`    | Copie des fichiers (et des dossiers avec `-r`) |
| `rm`     | `-r` `-f` | Supprime des fichiers (et des dossiers avec `-r`) |
| `mv`     |         | Déplace ou renomme des fichiers et des dossiers |
| `mkdir`  | `-p`    | Crée des répertoires (`-p` crée les parents) |
| `help`   |         | Affiche l'aide des commandes intégrées |
| `exit`   | `[code]`| Quitte le shell (avec un code de retour optionnel) |

Une commande inconnue produit : `Command '<nom>' not found`.

### Détails sur `ls -l`

Le format long reproduit la sortie de `ls` de coreutils : chaîne de
permissions, nombre de liens, propriétaire et groupe (résolus depuis
`/etc/passwd` et `/etc/group`), taille (ou numéros majeur/mineur pour les
fichiers spéciaux), date de modification en heure **locale**, puis le nom.

## Bonus implémentés

- **Ctrl+C (SIGINT)** géré proprement : le shell ne se ferme pas
- **Chaînage de commandes** avec `;` (ex. `mkdir a ; cd a ; pwd`)
- **Invite contextuelle** affichant le répertoire courant (`$HOME` abrégé en `~`)
- **Sortie colorée** pour `ls` (uniquement sur un vrai terminal)
- **Variables d'environnement** (`$HOME`, `${USER}`, …)
- **Guillemets et échappements** : simples `'…'`, doubles `"…"`, `\`
- **Commande `help`** documentant les fonctionnalités

## Architecture

```text
src/
├── main.rs            Boucle REPL, gestion de l'EOF, invite contextuelle
├── lexer.rs           Découpage : guillemets, échappements, expansion de $VAR, chaînage ;
├── signal.rs          Gestion du Ctrl+C via FFI libc (sans crate)
├── timefmt.rs         Formatage de l'heure locale pour ls -l (localtime_r via FFI)
├── userdb.rs          Résolution uid/gid -> nom via /etc/passwd et /etc/group
├── color.rs           Coloration de la sortie selon le type de fichier
└── commands/          echo, cd, pwd, ls, cat, cp, rm, mv, mkdir, help, exit
```

> Remarque : `signal.rs` et `timefmt.rs` utilisent des appels FFI directs vers
> la libc (`signal`, `localtime_r`) — déjà liée au programme — ce qui évite
> toute dépendance externe tout en restant fidèle au comportement Unix.

## Contraintes respectées

- Aucun binaire externe ni appel système qui en lance
- Aucune dépendance de *crate* (bibliothèque standard uniquement)
- Pas de pipes `|`, de redirections `>` ni de *globbing* `*` (hors périmètre)

## Licence

Projet réalisé dans le cadre du cursus [01-edu / Zone01](https://01-edu.org/).
