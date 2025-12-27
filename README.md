# Vector Bot (Rust)

Ce projet est une base solide et optimisée pour un bot Discord écrit en Rust, utilisant le framework [Poise](https://github.com/serenity-rs/poise) (basé sur [Serenity](https://github.com/serenity-rs/serenity)).

## Fonctionnalités
- **Architecture propre** : Structure modulaire pour ajouter facilement de nouvelles commandes.
- **Commandes Slash** : Support natif des commandes slash (`/`).
- **Configuration simple** : Utilisation de variables d'environnement (`.env`).
- **Performance** : Construit sur l'écosystème asynchrone Rust (Tokio).

## Prérequis

Avant de commencer, assurez-vous d'avoir installé **Rust** et **Cargo** sur votre machine.
Si ce n'est pas le cas, installez-les via [rustup.rs](https://rustup.rs/) :

```bash
# Sur Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Sur Windows
# Téléchargez et lancez rustup-init.exe depuis le site officiel.
```

### Problème courant sur Windows : `linker 'link.exe' not found`

Si vous rencontrez l'erreur `linker 'link.exe' not found` lors de la compilation, c'est qu'il vous manque les outils de compilation C++.

**Solution :**
1. Téléchargez et installez **Visual Studio Build Tools** (ou Visual Studio Community).
2. Lors de l'installation, cochez la case **"Développement Desktop en C++"** (Desktop development with C++).
3. Laissez l'installation se terminer et redémarrez votre terminal (ou votre PC).

## Installation

1. **Cloner le projet** (si ce n'est pas déjà fait) :
   ```bash
   git clone <votre-repo>
   cd vector_bot
   ```

3. **Configuration** :
   - Créez un fichier nommé `.env` à la racine du projet (à côté de `Cargo.toml`).
   - Copiez le contenu de `.env.example` ou ajoutez simplement votre token et l'ID de votre serveur :
     ```env
     DISCORD_TOKEN=votre_token_discord_ici
     DISCORD_GUILD_ID=votre_id_serveur_ici
     ```
     > **Pour trouver l'ID de votre serveur** : Activez le mode développeur dans les paramètres Discord (Avancé > Mode développeur), puis faites un clic droit sur l'icône de votre serveur > "Copier l'identifiant".
   > **IMPORTANT** : Ne partagez jamais votre fichier `.env` et ne le commitez jamais sur Git (il est déjà ignoré par `.gitignore`).

3. **Compiler le projet** :
   ```bash
   cargo build
   ```
   *La première compilation peut prendre un peu de temps car Rust compile toutes les dépendances.*

## ▶Lancement

Pour lancer le bot :

```bash
cargo run
```

Si tout fonctionne, vous verrez :
- `Enregistrement des commandes slash...`
- `Le bot est prêt ! Connecté en tant que [NomDuBot]`

Vous pourrez alors aller sur votre serveur Discord et taper `/hello` pour tester !

## Notes Importantes

- **Intents** : Ce bot utilise actuellement les intents non privilégiés (`GatewayIntents::non_privileged()`). Si vous avez besoin de lire le contenu des messages (pour l'ancien style de commandes) ou de détecter les membres qui rejoignent, vous devrez activer les "Privileged Gateway Intents" sur le [Portail Développeur Discord](https://discord.com/developers/applications) et modifier `src/main.rs`.

## Documentation

- [Guide Poise](https://docs.rs/poise/latest/poise/)
- [Documentation Serenity](https://docs.rs/serenity/latest/serenity/)
- [Livre Rust (The Rust Book)](https://doc.rust-lang.org/book/)
