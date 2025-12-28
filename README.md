# Vector Bot

Un bot Discord multifonction développé en Rust avec le framework Poise et la librairie Serenity.

## Fonctionnalités

### Système de Tickets

- **Création interactive** :
  - Déclenchement par message privé (DM) au bot.
  - Choix de la langue (Français/Anglais).
  - Choix de la catégorie (Partenariat, Recrutement, Support, Autres).
  - Création automatique d'un salon privé sur le serveur.
  - Permissions configurées automatiquement (Staff + Utilisateur + Bot).

- **Gestion** :
  - `/rep <message>` : Permet au staff de répondre à l'utilisateur de manière anonyme ("Support: Message").
  - `/close` : Ferme le ticket, supprime le salon, envoie un transcript complet (fichier .txt) dans les logs et notifie l'utilisateur.
  - `/rename <nom>` : Permet de renommer le salon du ticket.

- **Automatisation** :
  - Relance automatique par DM après 24h d'inactivité.
  - Fermeture automatique après 48h d'inactivité.

- **Modération des Tickets** :
  - `/blticket @user <raison>` : Blacklist un utilisateur (l'empêche d'ouvrir des tickets).
  - `/unblticket @user` : Retire un utilisateur de la blacklist.
  - Vérification automatique à chaque message privé.

### Système Vocal (Join to Create)

- **Création Automatique** :
  - Rejoindre un salon "Hub" (configuré dans `config.toml`) crée un salon vocal temporaire.
  - Nom du salon : `🔉〢Pseudo`.
  - Le créateur devient propriétaire.

- **Gestion** :
  - Le propriétaire a les permissions de modération sur son salon (Mute, Deafen, Move, Manage Channels).
  - Suppression automatique du salon quand il est vide.
  - Transfert automatique de propriété si le propriétaire quitte (mais qu'il reste du monde).

- **Logs** :
  - Logs de création, suppression et transfert de propriété dans un salon dédié.
  - Détection et log si un salon est supprimé manuellement.

### Commandes Utilitaires

- `/info` : Affiche les statistiques du bot (Uptime, Latence, RAM, CPU, Tickets actifs, Blacklists).
- `/profil [@user]` : Affiche le profil d'un utilisateur (Date création, Date arrivée, Statut Staff/Client, Statut Blacklist).
- `/clear <nombre>` : Supprime un nombre défini de messages (max 99).
- `/hello` : Commande de test basique.

## Configuration

Le bot se configure via le fichier `config.toml` :

```toml
[roles]
staff_role_id = 123456789...
client_role_id = 123456789...

[channels]
log_channel_id = 123456789...
voice_log_channel_id = 123456789...
jtc_channel_ids = [123456789..., 987654321...]

[categories]
partnership = 123456789...
recruitment = 123456789...
support = 123456789...
other = 123456789...
voice_category_id = 123456789...
```

## Base de Données

Le bot utilise SQLite (`database.db`) pour stocker :
- Les tickets actifs.
- La blacklist des tickets.
- Les salons vocaux temporaires.
- Les compteurs de tickets.

## Installation et Lancement

1. Cloner le dépôt.
2. Créer un fichier `.env` avec :
   ```
   DISCORD_TOKEN=votre_token
   DISCORD_GUILD_ID=votre_id_serveur
   ```
   > Pour trouver l'ID de votre serveur : Activez le mode développeur dans les paramètres Discord (Avancé > Mode développeur), puis faites un clic droit sur l'icône de votre serveur > "Copier l'identifiant".
   
   > **IMPORTANT** : Ne partagez jamais votre fichier `.env` et ne le commitez jamais sur Git (il est déjà ignoré par `.gitignore`).

3. Configurer `config.toml`.
4. Lancer avec `cargo run`.

## Notes Importantes

- **Intents** : Ce bot utilise actuellement les intents non privilégiés (`GatewayIntents::non_privileged()`). Si vous avez besoin de lire le contenu des messages (pour l'ancien style de commandes) ou de détecter les membres qui rejoignent, vous devrez activer les "Privileged Gateway Intents" sur le [Portail Développeur Discord](https://discord.com/developers/applications) et modifier `src/main.rs`.

## Documentation

- [Guide Poise](https://github.com/serenity-rs/poise)
- [Documentation Serenity](https://docs.rs/serenity/latest/serenity/)
- [Livre Rust (The Rust Book)](https://doc.rust-lang.org/book/)

## Problème courant sur Windows : `linker 'link.exe' not found`

Si vous rencontrez l'erreur `linker 'link.exe' not found` lors de la compilation, c'est qu'il vous manque les outils de compilation C++.

**Solution :**

1. Téléchargez et installez Visual Studio Build Tools (ou Visual Studio Community).
2. Lors de l'installation, cochez la case "Développement Desktop en C++" (Desktop development with C++).
3. Laissez l'installation se terminer et redémarrez votre terminal (ou votre PC).
