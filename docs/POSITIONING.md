# Positionnement de Marklight

Revue datée du 19 septembre 2026. Les faits concernant Marklight viennent du dépôt `v0.1.1` ; les autres produits sont cités uniquement quand une source officielle ou un essai local le permet. Ce document décrit des usages et des limites, sans classement ni revendication de performance comparative.

## Pour qui et pour quoi

Marklight s'adresse à quelqu'un qui **lit** des fichiers Markdown locaux et veut rester dans son terminal ou passer dans une fenêtre dédiée sans ouvrir un éditeur. Son contrat est volontairement restreint : fichiers UTF-8 bornés, rendu local, pas de compte, télémétrie, édition ou récupération réseau. Les liens HTTP(S) ne sont ouverts par le système qu'après un clic ; les images distantes sont bloquées. Voir [architecture](ARCHITECTURE.md), [dialecte](MARKDOWN.md) et [sécurité](../SECURITY.md).

| Parcours | Essai reproductible dans ce dépôt | Ce que l'on attend de Marklight |
|---|---|---|
| Lire un README en terminal | `marklight README.md` ou `marklight fixtures/markdown/gfm.md --plain --no-pager` | Rendu des titres, listes, tableaux et blocs de code, largeur adaptée et pager pour un long texte. |
| Lire un guide local à plusieurs pages | `marklight open fixtures/markdown/links.md`, puis suivre `Local` vers `basic.md` | Fenêtre de lecture, sommaire, recherche dans le document, liens locaux, sélection et rechargement après sauvegarde. Le retour à la page précédente n'existe pas encore en 0.1.1. |
| Lire la sortie d'un outil | `cat fixtures/markdown/gfm.md | marklight - --plain --no-pager` | Une sortie terminal sans compte ni fichier intermédiaire. Le bureau exige un fichier enregistré. |

Ces commandes ont été exécutées pour les parcours CLI sur macOS arm64 avec `marklight 0.1.1`. Le parcours bureau est décrit par le code et les tests de [validation](VALIDATION.md) ; cette revue de positionnement n'est pas un nouvel essai natif de navigation multi-fichiers.

## Alternatives pertinentes

| Produit | Recoupement documenté ou constaté | Différence de périmètre établie | Portée de la preuve |
|---|---|---|---|
| [Glow](https://github.com/charmbracelet/glow/blob/main/README.md) | Lecture terminal de fichier et stdin, largeur configurable, TUI de découverte de Markdown et pager. | Marklight ajoute une fenêtre Tauri dédiée ; Glow documente aussi des sources distantes et des paquets sur davantage d'OS. | Le binaire officiel **Glow 3.0.0 macOS arm64** a rendu ici `fixtures/markdown/gfm.md` depuis un chemin et stdin avec `-w 80`. Sa TUI et ses paquets d'autres OS n'ont pas été testés. Le [lecteur TUI](https://github.com/charmbracelet/glow/blob/main/ui/pager.go) de la branche consultée ne prouve pas le comportement de toutes les releases. |
| [Aperçu Markdown de VS Code](https://code.visualstudio.com/docs/languages/markdown) | Un `.md` peut être prévisualisé, notamment à côté de la source, et l'aperçu suit les modifications. | L'aperçu est une fonction de l'éditeur VS Code ; le lecteur Marklight est une application séparée et offre aussi un CLI de rendu. | **VS Code 1.138.0 macOS arm64** installé sur ce poste : `gfm.md` a été ouvert puis « Open Preview to the Side » a affiché titres, listes, code et tableau. Cette observation ne mesure ni ressources, ni qualité du rendu, ni expérience sur d'autres OS. |
| [MarkText](https://github.com/marktext/marktext/blob/develop/README.md) | Son éditeur graphique documente ouverture de fichiers locaux, aperçu, recherche et sommaire. | La [commande documentée](https://marktext.me/docs/cli) ouvre l'application ; elle n'est pas une sortie Markdown dans le terminal. MarkText couvre aussi l'édition et l'export. | **Documentation seulement** : aucun binaire MarkText n'a été installé ou comparé ici. Les capacités de la branche `develop` et celles d'un paquet donné peuvent différer. |

### Méthode et conclusions permises

Glow et Marklight ont été exécutés sur **le même fichier** `fixtures/markdown/gfm.md`, depuis un chemin et stdin. Les deux sorties contenaient le titre, le code et la liste ; leurs mises en forme différentes n'ont pas été notées ni classées. Pour VS Code, le même fichier a été ouvert dans la version installée et son aperçu latéral observé. Ces essais ne couvrent pas la recherche interne de Glow, la navigation entre fichiers, les écrans natifs de Marklight ou MarkText. Il serait donc infondé de dire que Marklight est plus rapide, plus fidèle, plus accessible ou meilleur que ces alternatives.

Le choix produit qui ressort du dépôt est **lecture locale dans deux environnements avec un moteur Rust commun**. Sa valeur dépendra surtout de la fluidité sur les vrais guides, d'un historique de navigation, d'une installation sans friction et de preuves natives répétables. Ajouter édition, synchronisation ou récupération distante diluerait ce contrat et élargirait sa frontière de sécurité ; aucune de ces fonctions n'est engagée par cette étude.

Avant de publier un tableau concurrentiel dans le README ou le site, refaire les essais sur les **versions binaires exactes**, le même dossier multi-fichiers et les OS annoncés ; relever ouverture, sommaire, recherche dans le contenu, liens, copie et rechargement. Une revue par un utilisateur qui n'a pas participé au développement reste également à faire. Le tableau ci-dessus est un état de preuve, pas une publicité comparative.
