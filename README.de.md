# Rust Git CLI

Ein intelligentes Git-Commit-Tool, das mit KI zweisprachige (Chinesisch/Englisch) Commit-Nachrichten generiert.

## Funktionen

- **KI-gesteuert** - Unterstützt OpenAI, Anthropic und benutzerdefinierte Endpunkte (z.B. DeepSeek)
- **Zweisprachige Commits** - Generiert automatisch Chinesisch/Englische Commit-Nachrichten nach Conventional Commits
- **Intelligentes Staging** - Erkennt nicht gestagete Änderungen und fordert zur Bestätigung auf
- **Interaktive Benutzeroberfläche** - Farbige Ausgabe, Diff-Vorschau, Commit-Bestätigung
- **Flexible Konfiguration** - Mehrstufige Konfigurationsdateien und Umgebungsvariablen

## Installation

```bash
# Aus Quellcode bauen
git clone https://github.com/duanyongcheng/rust-git-cli.git
cd rust-git-cli
cargo build --release

# Im System installieren
cargo install --path .
```

**Voraussetzungen**: Rust 1.70+, Git 2.0+

## Schnellstart

### 1. Konfiguration initialisieren

```bash
rust-git-cli init                 # Globale Konfiguration erstellen (~/.config/rust-git-cli/config.toml)
rust-git-cli init --local         # Projektkonfiguration erstellen (.rust-git-cli.toml)
```

### 2. API-Schlüssel einrichten

```bash
# Empfohlen: Umgebungsvariablen verwenden
export OPENAI_API_KEY="your-api-key"
# oder
export ANTHROPIC_API_KEY="your-api-key"
```

### 3. Verwendung

```bash
rust-git-cli                      # Repository-Status prüfen (Standard)
rust-git-cli commit               # KI-Commit-Nachricht generieren
rust-git-cli commit --show-diff   # Diff vor Generierung anzeigen
rust-git-cli commit --debug       # Debug-Modus
```

## Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `status` | Repository-Status prüfen (Standard) |
| `commit` | KI-Commit generieren und ausführen |
| `diff` | Code-Änderungen anzeigen |
| `log` | Commit-Verlauf anzeigen |
| `init` | Konfigurationsdatei initialisieren |

### commit Optionen

```bash
rust-git-cli commit [OPTIONS]

Optionen:
  --api-key <KEY>      API-Schlüssel temporär angeben
  --model <MODEL>      KI-Modell angeben (z.B. gpt-4, deepseek-v3)
  --base-url <URL>     Benutzerdefinierter API-Endpunkt
  --auto               Bestätigung überspringen und direkt committen
  --show-diff          Diff vor Generierung anzeigen
  --debug              Rohe KI-Antwort anzeigen
```

### log Optionen

```bash
rust-git-cli log [OPTIONS]

Optionen:
  -n, --count <N>      Anzahl der anzuzeigenden Commits (Standard: 10)
  --grep <PATTERN>     Nach Inhalt filtern
  --author <NAME>      Nach Autor filtern
  --since <DATE>       Startdatum (z.B. "2024-01-01" oder "1 week ago")
  --until <DATE>       Enddatum
  --full               Vollständige Commit-Nachricht anzeigen
```

### diff Optionen

```bash
rust-git-cli diff [OPTIONS]

Optionen:
  --staged             Nur gestagete Änderungen anzeigen
```

## Konfiguration

Suchreihenfolge für Konfigurationsdateien:
1. `./.rust-git-cli.toml` (Projektebene)
2. `~/.config/rust-git-cli/config.toml` (Benutzerebene)
3. `~/.rust-git-cli.toml` (Benutzerebene Fallback)

### Beispielkonfiguration

```toml
[ai]
provider = "openai"                      # openai oder anthropic
model = "gpt-4"                          # Modellname
api_key_env = "OPENAI_API_KEY"           # Name der API-Schlüssel-Umgebungsvariable
# api_key = "sk-..."                     # Direkte Einstellung (nicht empfohlen)
# base_url = "https://api.deepseek.com/v1"  # Benutzerdefinierter Endpunkt
max_tokens = 2000

[commit]
max_diff_size = 4000                     # Maximale Diff-Zeichen, die an KI gesendet werden
auto_stage = false                       # Alle Änderungen automatisch stagen
```

### API-Schlüssel Priorität

1. Kommandozeilenargument `--api-key`
2. Konfigurationsdatei `api_key`
3. Umgebungsvariable (angegeben durch `api_key_env`)
4. Interaktive Eingabe

## Commit-Nachrichten-Format

Generierte Commit-Nachrichten folgen der [Conventional Commits](https://www.conventionalcommits.org/) Spezifikation:

```
feat(auth): 添加用户认证功能
Add user authentication feature

实现了JWT令牌验证
Implement JWT token validation
添加了用户登录接口
Add user login endpoint
```

### Commit-Typen

| Typ | Beschreibung |
|-----|--------------|
| `feat` | Neues Feature |
| `fix` | Fehlerbehebung |
| `docs` | Dokumentation |
| `style` | Code-Formatierung |
| `refactor` | Code-Refactoring |
| `test` | Tests |
| `chore` | Build/Tooling |
| `perf` | Performance |

## Arbeitsablauf

```bash
# 1. Status prüfen
$ rust-git-cli status

# 2. Änderungen anzeigen
$ rust-git-cli diff

# 3. Commit generieren
$ rust-git-cli commit

# Wenn nicht gestagete Änderungen erkannt werden:
# Unstaged changes detected:
# ──────────────────────────────────────────────────
#   M src/main.rs
#   ? src/new_file.rs
# ──────────────────────────────────────────────────
# Do you want to stage all changes (git add .)? (Y/n)

# Nach KI-Generierung, Aktion wählen:
# - Accept and commit: Akzeptieren und committen
# - Edit message: Vor dem Commit bearbeiten
# - Regenerate: Neu generieren
# - Cancel: Abbrechen
```

## Fehlerbehebung

| Problem | Lösung |
|---------|--------|
| API-Verbindung fehlgeschlagen | Netzwerk prüfen, API-Schlüssel verifizieren, `--debug` für Details |
| JSON-Parsing-Fehler | `--debug` für Rohantwort, anderes Modell versuchen |
| Konfiguration funktioniert nicht | Dateipfad und TOML-Format prüfen |
| Commit fehlgeschlagen | Git-Benutzer konfiguriert sicherstellen (`git config user.name/email`) |

```bash
# Debug-Modus
rust-git-cli commit --debug

# Hilfe anzeigen
rust-git-cli --help
rust-git-cli commit --help
```

## Entwicklung

```bash
cargo build                       # Bauen
cargo test                        # Testen
cargo fmt                         # Formatieren
cargo clippy -- -D warnings       # Linten
```

### Projektstruktur

```
src/
├── main.rs          # Einstiegspunkt und Befehlsverteilung
├── cli.rs           # CLI-Definitionen (clap)
├── config.rs        # Konfigurationsverwaltung
├── git.rs           # Git-Operationen (git2)
├── ui.rs            # Interaktive Benutzeroberfläche (dialoguer)
└── ai/
    ├── mod.rs       # KI-Client-Abstraktion
    ├── openai.rs    # OpenAI-Implementierung
    └── anthropic.rs # Anthropic-Implementierung
```

## Lizenz

MIT License

## Danksagungen

- [git2-rs](https://github.com/rust-lang/git2-rs) - Git-Operationen
- [clap](https://github.com/clap-rs/clap) - Kommandozeilen-Parsing
- [dialoguer](https://github.com/console-rs/dialoguer) - Interaktive Benutzeroberfläche
- [colored](https://github.com/colored-rs/colored) - Farbige Ausgabe
