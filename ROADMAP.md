# Roadmap de Marklight — audit du dépôt et chemin vers une version de référence

État examiné le **19 septembre 2026** sur `main` (`763e28b`, tag local `v0.1.1`). Ce document est un plan, pas une liste de fonctionnalités livrées. Les priorités **P0** conditionnent la prochaine publication crédible ; les **P1** élargissent l'utilité et l'adoption. Chaque phase se termine par des preuves datées dans `docs/VALIDATION.md`. Une case ne sera cochée que lorsque son critère aura été vérifié sur l'artefact concerné.

## 1. Ce que le dépôt démontre aujourd'hui

### Produit et proposition de valeur

Marklight est un **lecteur de Markdown local**, utilisable dans le terminal et dans une fenêtre Tauri. Le CLI et l'application partagent les événements Markdown de `marklight-core` ; `marklight-render` produit une sortie terminal et un HTML filtré. La promesse « lire un README ou un guide sans passer en mode édition » est compréhensible. Le périmètre assumé est bon : pas de compte, de télémétrie, de serveur installé, d'éditeur ou d'exécution du contenu. Voir `README.md`, `docs/ARCHITECTURE.md`, `docs/MARKDOWN.md` et `SECURITY.md`.

La valeur distinctive **possible** est la combinaison terminal + lecteur natif, avec rendu Rust commun, navigation dans les titres, recherche, sélection/copie, rechargement après sauvegarde et lecture hors ligne. Le dépôt ne contient toutefois ni étude d'usage, ni comparaison testée avec d'autres lecteurs, ni mesure de conversion/adoption. Il ne prouve donc aucune supériorité sur un concurrent, et le nombre d'étoiles ne peut pas être promis. Les familles à comparer lors de l'exécution du plan sont les aperçus d'éditeurs, les lecteurs Markdown en terminal et les lecteurs de documents dédiés ; leurs capacités doivent être vérifiées séparément avant toute affirmation publique.

### Acquis vérifiés et limites de preuve

| Domaine | Présent dans le dépôt ou vérifié ici | Limite actuelle |
|---|---|---|
| Fonctions | Fichiers/dossiers/entrée standard, sortie simple et pager ; fenêtre native avec sommaire, recherche, thèmes, taille de texte, mode zen, récents, liens Markdown relatifs, images raster locales autorisées et rechargement. Les chemins passent par `crates/marklight-*` et `apps/desktop/`. | Pas d'historique précédent/suivant lors du parcours de plusieurs fichiers. Math, Mermaid, wiki links, front matter et notes de bas de page sont déclarés non interprétés dans `docs/MARKDOWN.md` ; leur nécessité reste à trancher. |
| Code et sécurité | Séparation core/render/CLI/Tauri, limites de taille, filtrage HTML, URL autorisées, CSP et tests de documents hostiles. `cargo fmt --check`, Clippy strict hors ligne et `cargo test --workspace --offline` passent le 19/09/2026 : **21 tests Rust**. | Ces tests ne constituent ni un audit de sécurité complet, ni une preuve contre les courses de fichiers ou tous les Markdown malveillants. `main.ts`, `reader.rs` et le processus de rendu restent des points de complexité à mesurer avant refonte. |
| Interface | Design cohérent dans les captures claires, sombres et étroites ; lecture et contrôles visibles. Build TypeScript/Vite et **9 tests Playwright** passent ici. | Les captures versionnées de `docs/images/` utilisent le rendu Rust et un adaptateur IPC de test (`docs/VALIDATION.md`) : elles ne sont pas des captures du binaire natif installé. Aucun audit complet VoiceOver/NVDA ni geste Finder de dépôt vérifié n'est documenté. |
| Performance | `docs/PERFORMANCE.md` et `docs/measurements.json` publient méthode, matériel et limites. Les documents de 10 Mo atteignent le rendu natif. | Mesures de la version 0.1.0 : environ **11 s** jusqu'à la disponibilité DOM/layout à 10 Mo, avec un échantillon transitoire d'environ **799 Mio RSS** ; pas de mesure actuelle 0.1.1 ni de garantie de fluidité. Le document et le sommaire sont encore matérialisés en entier ; la recherche marque jusqu'à 10 000 occurrences. L'objectif de démarrage natif à 500 ms n'était pas atteint. |
| Installation et distribution | Guides, tags Git locaux `v0.1.0` et `v0.1.1`, scripts de packaging, archives/DMG/checksums locaux ignorés par Git ; paquet npm arm64 macOS décrit dans `docs/VALIDATION.md`. | Les liens GitHub Releases, npm et Pages figurent dans les docs, mais leur état **actuel** et l'installation d'un téléchargement public n'ont pas été revérifiés dans cet audit du dépôt. Le paquet npm ne contient que le CLI macOS arm64. App macOS à signature ad hoc, sans notarisation ; Intel Mac, Windows et Linux non validés. Crates.io non publié selon les docs. |
| Automatisation et présentation | `scripts/check.sh`, tests Rust/Playwright, contrôle du site, README, captures, site et changelog existent. `node scripts/check-site.mjs` passe ici (390/1280 px, références locales, copies et console). | Aucun workflow `.github/workflows` n'est versionné ; `scripts/check.sh` échoue volontairement si on en ajoute un. Pas de vidéo de démonstration dans l'arbre suivi. Les badges SVG sont locaux et ne démontrent pas l'état courant d'une release/CI. Pas de modèles d'issues/PR versionnés. |

**Frontière de cet audit :** les validations ci-dessus ont été lancées sur ce poste, sauf les mesures historiques et essais natifs cités comme tels. Aucun nouveau paquet, téléchargement public, test sur autre OS, publication, configuration GitHub ou comparaison externe n'a été réalisé pour écrire ce plan. `artifacts/`, `target/` et `dist/` sont ignorés par Git ; leur présence locale ne constitue pas une preuve qu'un visiteur peut télécharger ces fichiers.

## 2. Règles de sortie du roadmap

1. Conserver la frontière produit : un lecteur rapide, sûr et local. N'ajouter des formats ou fonctions que s'ils servent un scénario de lecture réel et mesuré.
2. Séparer dans `docs/VALIDATION.md` : **test local passé**, **test natif passé sur un système nommé**, **artefact public installé et testé**, **plateforme non vérifiée**. Mettre à jour les dates et versions après chaque changement important.
3. Avant une release, exécuter les mêmes portes sur le commit et les binaires publiés. Un build, un tag local, une capture de l'adaptateur IPC ou un checksum généré ne prouvent pas seuls une distribution utilisable.
4. Les documents actuels (`docs/PLAN.md`, `CONTRIBUTING.md`, `scripts/check.sh`) interdisent intentionnellement les GitHub Actions sans décision du mainteneur. La phase d'automatisation comprend cette décision explicite et l'alignement de ces fichiers ; ce plan ne modifie pas ce réglage.
5. La **phase vidéo est la dernière**. Elle démarre seulement quand toutes les tâches et validations des phases précédentes sont terminées, y compris la vérification du produit livré.

## Suivi d'exécution

Une case cochée représente la tâche entière et ses validations, pas seulement
un fichier écrit. État au 19 septembre 2026 :

- [ ] **0.1** Positionnement et parcours : [comparaison bornée](docs/POSITIONING.md)
  rédigée, essais locaux Glow/VS Code faits ; revue par une personne extérieure
  et comparaison pratique de MarkText encore absentes.
- [x] **0.2** [Baseline 0.1.1](docs/PERFORMANCE.md) datée, gates locaux
  réussis, [mesures brutes](docs/measurements-0.1.1-baseline.json) et
  [contrat de release](docs/RELEASE-CHECKLIST.md) publiés. Cela ne valide pas
  les téléchargements publics ou les autres OS.
- [ ] **1.1** Gros documents et mesures end-to-end : lookup des titres et
  sommaire fenêtré implémentés ; [profil expérimental](docs/PERFORMANCE.md#phase-1-development-profile-on-2026-09-19)
  enregistré. Cible 10 Mo, interactions, guide varié et mémoire complète encore
  non validés.
- [ ] **1.2** Recherche par morceaux avec annulation, plafond exact et sommaire
  filtrable implémentés ; tests Chromium et essai natif à 100 Ko passés.
  Réactivité native mesurée à 1/10 Mo et audit lecteur d'écran encore absents.
- [x] **1.3** Historique, rechargement et erreurs : retour/avance avec position
  restaurée, alerte persistante et reprise, réponse obsolète écartée ;
  [tests et essai WebKit natif](docs/VALIDATION.md#development-progress-after-v011-on-2026-09-19).
- [ ] **1.4** Dialecte Markdown choisi et testé.
- [ ] **2.1** Frontière des documents non fiables.
- [ ] **2.2** Gestes natifs et accessibilité.
- [ ] **2.3** Architecture et régressions.
- [ ] **3.1** Première utilisation.
- [ ] **3.2** États visuels réels.
- [ ] **4.1** Porte de qualité et décision CI.
- [ ] **4.2** Plateformes installées et validées.
- [ ] **4.3** Confiance macOS et registres.
- [ ] **4.4** Artefacts traçables.
- [ ] **5.1** README et captures natives.
- [ ] **5.2** Site et documentation alignés.
- [ ] **5.3** Adoption et contributions.
- [ ] **5.4** Release publiée et testée depuis téléchargement.
- [ ] **6.1** Captures d'usage réel après toutes les autres phases.
- [ ] **6.2** Montage, exports et lecture intégrale vérifiée.

## Phase 0 — Positionnement et référence reproductible (P0)

### 0.1 — Définir le parcours qui mérite d'être partagé

- **Objectif :** montrer en une minute pourquoi un développeur ouvrirait Marklight plutôt qu'un aperçu dans son éditeur ou une commande de lecture brute.
- **Changements :** décrire 2 à 3 cas réels (README de dépôt, documentation locale à plusieurs pages, texte reçu sur stdin) ; dresser une matrice de besoins où chaque ligne est démontrable ; faire plus tard une évaluation pratique datée de projets concurrents avant toute comparaison publique. Décider et documenter ce qui reste hors périmètre (édition, rendu réseau, synchronisation).
- **Parties concernées :** `README.md`, `site/index.html`, `docs/PLAN.md`, nouveau `docs/POSITIONING.md` ou équivalent.
- **Acceptation :** proposition de valeur et public cible explicites ; chaque bénéfice renvoie à une commande, un fichier d'exemple et une preuve ; aucune affirmation de leadership/performance relative sans protocole comparatif.
- **Validation :** revue des parcours par une personne qui n'a pas écrit le projet ; reprise des commandes et comparaison sur les mêmes fichiers, systèmes et versions si un concurrent est nommé.
- **Dépendances/risques :** l'étude concurrentielle sort du seul dépôt et doit être faite lors de l'exécution ; risque de multiplier des fonctions étrangères à la lecture.

### 0.2 — Geler une base de validation et un contrat de release

- **Objectif :** savoir exactement ce qui change et ce qu'on peut annoncer pour la prochaine version.
- **Changements :** relever commit/versions, sorties de `scripts/check.sh`, benchs petit/gros fichiers et limites connues ; définir la matrice minimale de plateformes et les budgets d'interaction après mesure ; créer une checklist release avec responsables des comptes, signatures et tests natifs.
- **Parties concernées :** `docs/VALIDATION.md`, `docs/PERFORMANCE.md`, `docs/measurements.json`, `scripts/benchmark.py`, `Cargo.toml`, manifestes npm/Tauri.
- **Acceptation :** baseline datée et reproductible sur le commit choisi ; écart 0.1.0/0.1.1 explicite ; cibles chiffrées justifiées par les mesures et nom des plateformes réellement supportées.
- **Validation :** exécuter formatage, Clippy, tests, build, tests Playwright, smoke pager/site et benchmarks hors compilation simultanée ; enregistrer commandes, versions et résultats bruts.
- **Dépendances/risques :** priorité préalable à l'optimisation ; mesures sensibles au cache, au WebView et au matériel.

## Phase 1 — Rendre la lecture robuste sur des documents réels (P0)

### 1.1 — Réduire le coût des gros documents sans perdre le contenu

- **Objectif :** garder ouverture, défilement et navigation utilisables sur des README et guides denses.
- **Changements :** profiler séparément chargement, parsing, HTML, IPC, assemblage DOM, sommaire et paint ; limiter le travail bloquant du WebView ; choisir ensuite une stratégie de rendu progressif/virtualisation ou de sommaire par fenêtre, avec accès aux titres, recherche, sélection, copie et ancres hors écran. Éviter une réécriture avant profilage.
- **Parties concernées :** `apps/desktop/src/main.ts`, `apps/desktop/src/reader.css`, `apps/desktop/src-tauri/src/reader.rs`, `crates/marklight-render/src/html.rs`, `scripts/benchmark.py`.
- **Acceptation :** sur les corpus 1/5/10 Mo et sur un guide varié, les temps et la mémoire end-to-end sont publiés avant/après ; aucune interaction essentielle n'est perdue ; la cible adoptée en 0.2 est tenue ou le blocage est documenté.
- **Validation :** mesures natives répétées de démarrage, première interaction, recherche, clic de titre, défilement et RSS du **processus complet** ; tests de non-régression de sélection, code à copier, liens et rechargement.
- **Dépendances/risques :** 0.2 ; la virtualisation peut casser position de lecture, recherche et accessibilité.

### 1.2 — Faire évoluer recherche et sommaire à grande échelle

- **Objectif :** éviter que chercher un terme ou parcourir des milliers de titres bloque la fenêtre.
- **Changements :** mesurer `search.ts` (parcours complet, regroupement, marquage et nettoyage), le tableau `matches` et la création du sommaire ; introduire annulation, rendu progressif ou index adapté au volume ; rendre explicite le plafond des 10 000 résultats ; conserver la recherche à travers les styles inline.
- **Parties concernées :** `apps/desktop/src/search.ts`, `apps/desktop/src/main.ts`, `apps/desktop/tests/reader.spec.ts`, `fixtures/markdown/huge.md` et nouveaux corpus générés.
- **Acceptation :** saisie, annulation et passage au résultat suivant restent réactifs sur les cas de 0.2 ; nombre et état « résultats supplémentaires » justes ; les titres hors écran et les morceaux non encore peints restent accessibles.
- **Validation :** profil navigateur et natif, tests d'une recherche très fréquente, d'une phrase fractionnée entre balises, d'un changement de fichier pendant une recherche et de l'absence de gel prolongé.
- **Dépendances/risques :** 1.1 ; conserver exactitude et lecteur d'écran si le DOM est différé.

### 1.3 — Fiabiliser navigation, rechargement et erreurs

- **Objectif :** lire un ensemble de documents et revenir à son point de départ sans perdre son contexte.
- **Changements :** ajouter précédent/suivant avec historique de fichier, ancre et défilement ; préciser le comportement d'un lien relatif introuvable, d'un fichier supprimé ou d'une sauvegarde atomique ; conserver le dernier document valide et afficher une erreur persistante/actionnable ; éliminer les réponses obsolètes lors de changements rapides de fichier, thème ou watcher.
- **Parties concernées :** `apps/desktop/src/main.ts`, `apps/desktop/index.html`, `apps/desktop/src-tauri/src/reader.rs`, `crates/marklight-core/src/watch.rs`, tests Rust/Playwright.
- **Acceptation :** parcours A → B → retour restitue A et sa position ; échec d'ouverture ou de reload n'efface pas la lecture valide ; un événement ancien ne remplace jamais le dernier choix ; récents et historique ont une sémantique distincte documentée.
- **Validation :** scénarios de liens, suppression/recréation, sauvegarde atomique, ouverture rapprochée et dossier à plusieurs pages, dans le navigateur puis l'application native.
- **Dépendances/risques :** 1.1 si le rendu devient différé ; les comportements de watcher changent selon le système de fichiers.

### 1.4 — Décider le dialecte Markdown à couvrir

- **Objectif :** rendre les documents courants lisibles sans annoncer une compatibilité GitHub totale.
- **Changements :** constituer un corpus de README/guides représentatifs et une table « interprété / texte visible / bloqué » ; examiner les éléments déjà limités (`docs/MARKDOWN.md`) et choisir explicitement notes de bas de page, alertes, ancres ou autres syntaxes utiles. Ajouter seulement les cas retenus aux deux rendus, avec rendu sûr pour le HTML et dégradation claire dans le terminal.
- **Parties concernées :** `crates/marklight-core/src/document.rs`, `crates/marklight-render/src/{html,terminal}.rs`, `fixtures/markdown/`, `docs/MARKDOWN.md`.
- **Acceptation :** aucune syntaxe annoncée sans fixture et sortie vérifiée ; les éléments non supportés restent décrits honnêtement ; les corpus retenus conservent liens et copie fidèle.
- **Validation :** golden terminal, tests HTML et visuels, documents hostiles utilisant les nouvelles syntaxes.
- **Dépendances/risques :** 0.1 ; math/Mermaid peuvent introduire exécution ou téléchargements et doivent rester exclus si une implémentation locale sûre n'est pas démontrée.

## Phase 2 — Confiance, qualité technique et accessibilité (P0)

### 2.1 — Renforcer la frontière des documents non fiables

- **Objectif :** protéger l'ouverture d'un Markdown téléchargé ou fourni par un tiers.
- **Changements :** faire une revue ciblée du double filtrage HTML, des URL, des images, de la CSP et de la commande `follow_link` ; tester les substitutions de symlink/fichier entre autorisation et lecture, encodages, fragments, schémas et charges extrêmes ; corriger seulement les failles constatées. Ajouter une politique de mise à jour des dépendances et un contrôle d'avis de sécurité reproductible.
- **Parties concernées :** `crates/marklight-core/src/paths.rs`, `crates/marklight-render/src/html.rs`, `apps/desktop/src-tauri/src/{main,reader}.rs`, `SECURITY.md`, `Cargo.lock`, `apps/desktop/package-lock.json`.
- **Acceptation :** aucun test adversarial ne déclenche script, requête distante involontaire, lecture hors périmètre ou consommation illimitée ; signalement privé et limites du modèle de menace expliqués.
- **Validation :** fixtures/fuzzing borné sur parser et classificateurs, tests natifs de protocole image, audit de dépendances avec date et traitement de chaque alerte pertinente.
- **Dépendances/risques :** 1.4 ; certaines courses exigent des tests OS et une décision explicite sur les liens hors dossier. Aucun bug exploitable n'est présumé par cet audit.

### 2.2 — Vérifier les gestes natifs et le clavier

- **Objectif :** établir l'utilisabilité réelle du paquet, au-delà de l'adaptateur IPC du navigateur.
- **Changements :** automatiser autant que possible l'ouverture via Finder/OS, le dépôt de fichier, les menus/raccourcis, les liens, images, copie exacte, rechargement et récents ; effectuer une revue VoiceOver sur macOS et NVDA/lecteur adapté si Windows est ciblé ; réparer les problèmes de focus, noms de contrôles, contraste et annonce d'état trouvés.
- **Parties concernées :** `apps/desktop/index.html`, `apps/desktop/src/{main.ts,styles.css,reader.css}`, `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/tests/`, `docs/VALIDATION.md`.
- **Acceptation :** chemin complet ouvrable et navigable au clavier, focus visible, recherche et erreurs annoncées, clic/depot natif éprouvés ; liste explicite des environnements et aides techniques testés.
- **Validation :** checklist native enregistrée avec version de l'OS et du paquet, audit clavier et lecteur d'écran, Playwright comme régression complémentaire.
- **Dépendances/risques :** 1.3 ; l'adaptateur navigateur ne valide pas les dialogues, menus ou protocoles Tauri.

### 2.3 — Garder l'architecture simple et les régressions détectables

- **Objectif :** permettre des contributions ciblées sans casser les invariants du lecteur.
- **Changements :** isoler seulement les parties de `main.ts` et `reader.rs` devenues difficiles à tester après 1.1–2.2 (cycle d'ouverture, rendu, recherche, navigation, politique de fichiers) ; maintenir les contrats Rust/TypeScript ; rendre les fixtures déterministes et les erreurs diagnostiquables sans journaliser le contenu privé.
- **Parties concernées :** `apps/desktop/src/`, `apps/desktop/src-tauri/src/`, `crates/marklight-*/`, `fixtures/`, `docs/ARCHITECTURE.md`.
- **Acceptation :** responsabilité de chaque module documentée, aucune duplication du parsing dans les interfaces, tests de régression sur les flux déplacés et diff de comportement nul hors tâches prévues.
- **Validation :** suites Rust/TS/Playwright, Clippy strict, revue des chemins d'erreur et du contenu des logs.
- **Dépendances/risques :** effectuer après les changements fonctionnels ; risque de refactorisation sans bénéfice si elle n'est pas motivée par un test ou un profilage.

## Phase 3 — Première utilisation et qualité perçue (P1)

### 3.1 — Rendre les cinq premières minutes évidentes

- **Objectif :** obtenir une lecture réussie depuis une installation propre sans connaître Tauri, Cargo ou l'organisation du dépôt.
- **Changements :** proposer dans l'accueil un exemple local **réel**, empaqueté et lisible sans compte/réseau ; montrer « ouvrir un fichier », raccourcis et portée des liens/images ; rendre les erreurs d'installation du pont CLI → app et les chemins de récents lisibles et corrigibles.
- **Parties concernées :** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, ressources Tauri, `docs/INSTALLATION.md`, `README.md`.
- **Acceptation :** une personne sur système supporté installe le paquet, ouvre l'exemple puis son propre `.md` et retrouve les contrôles sans assistance ; le mode CLI seul reste clair.
- **Validation :** essais sur profil utilisateur vierge et paquet distribué, test des états vide/erreur/hors ligne, retour d'utilisateurs novices documenté.
- **Dépendances/risques :** phase 2 ; éviter d'empaqueter un faux écran ou un document qui masquerait l'absence de fichier réel.

### 3.2 — Polir les états réels à toutes les tailles

- **Objectif :** rendre l'interface aussi bonne en usage quotidien que dans les captures actuelles.
- **Changements :** vérifier fenêtres minimale et large, clair/sombre/système, document sans titres, tableaux et longs chemins, sommaire dense, images manquantes, chargement lent, fenêtre étroite et erreurs ; ajuster typographie, espacements, contraste, textes et actions selon les défauts observés ; garder une zone de lecture calme.
- **Parties concernées :** `apps/desktop/src/{styles.css,reader.css,main.ts}`, `apps/desktop/index.html`, `site/images/`, `docs/images/` après capture finale.
- **Acceptation :** aucun contrôle hors écran, chemin tronqué encore identifiable, état de chargement/action explicite, contrastes/focus vérifiés ; les captures publiées représentent le produit corrigé.
- **Validation :** captures et inspection native à plusieurs résolutions et thèmes, contrôle Playwright d'overflow/console, revue clavier et visuelle.
- **Dépendances/risques :** 1.1–1.3, 2.2 ; les captures de l'adaptateur ne suffisent pas pour juger les menus et le WebView natifs.

## Phase 4 — Automatisation et distribution installable (P0)

### 4.1 — Installer une porte de qualité reproductible

- **Objectif :** empêcher qu'un commit ou une release contourne les validations locales.
- **Changements :** consolider `scripts/check.sh` avec versions d'outils, exécution hors ligne lorsque pertinente, tests de site, packaging smoke et rapport lisible ; décider avec le mainteneur si les GitHub Actions doivent être activées. Si oui, retirer l'interdiction de `.github/workflows` du script, aligner `docs/PLAN.md`/`CONTRIBUTING.md`, ajouter des jobs séparés CLI, frontend, sécurité et packages par OS validé. Si non, documenter un gate manuel obligatoire et ses preuves attachées à chaque release.
- **Parties concernées :** `scripts/check.sh`, `scripts/check-site.mjs`, `CONTRIBUTING.md`, `docs/PLAN.md`, éventuellement `.github/workflows/`.
- **Acceptation :** une exécution fraîche sur le commit cible donne un verdict reproductible et visible ; aucun badge CI « passant » sans run réel ; politique CI cohérente entre scripts et docs.
- **Validation :** échec volontaire sur un test cassé, puis run réussi ; si workflow autorisé, vérifier les jobs sur un vrai commit/PR et leur état GitHub.
- **Dépendances/risques :** phases 1–3 ; décision du mainteneur requise car le dépôt interdit actuellement les workflows. Coût et différences des runners natifs.

### 4.2 — Élargir les plateformes seulement après essais natifs

- **Objectif :** rendre l'installation accessible au-delà d'un seul Mac sans promettre un système non testé.
- **Changements :** construire et exercer le CLI/app sur macOS arm64, puis Intel Mac, Windows x64 et Linux x64 selon les machines/runners réellement disponibles ; adapter scripts, associations `.md`, lanceur CLI → desktop et instructions d'installation ; produire uniquement les formats dont l'installation et la désinstallation ont été testées.
- **Parties concernées :** `scripts/package.sh`, `scripts/bundle-macos.py`, `apps/desktop/src-tauri/tauri.conf.json`, `crates/marklight-cli/src/main.rs`, `docs/INSTALLATION.md`, manifests de distribution.
- **Acceptation :** chaque plateforme annoncée possède un binaire avec nom/version/architecture, checksum, test de démarrage, ouverture `.md`, lecture et sortie ; les autres restent étiquetées « non vérifiées ».
- **Validation :** installation dans une machine propre de l'OS cible, tests natifs phase 2.2, contrôle des associations et de la désinstallation, conservation des journaux de build.
- **Dépendances/risques :** 4.1 ; accès aux OS requis. Une compilation croisée seule ne valide pas le WebView ni l'installateur.

### 4.3 — Réduire la friction de confiance macOS et des registres

- **Objectif :** offrir une voie d'installation fiable, avec une documentation exacte des limites.
- **Changements :** pour macOS, examiner signature Developer ID, notarisation et test Gatekeeper sur **DMG téléchargé**, sous réserve d'identité et de credentials du mainteneur ; garder les secrets hors dépôt. Vérifier disponibilité/ownership des crates avant d'envisager `cargo install marklight` ; décider si npm doit rester un binaire macOS arm64 ou devenir une distribution multi-plateforme cohérente, sans hooks opaques.
- **Parties concernées :** `scripts/bundle-macos.py`, `scripts/package-npm.sh`, `scripts/check-npm.py`, `packages/npm/`, `crates/*/Cargo.toml`, `docs/INSTALLATION.md`, `SECURITY.md`.
- **Acceptation :** les instructions installent effectivement la version annoncée ; signature/notarisation revendiquées seulement après vérification du paquet téléchargé ; noms des crates et disponibilité des plateformes exacts ; aucun secret dans Git, logs publics ou archives.
- **Validation :** vérification des signatures, notarisation et Gatekeeper sur téléchargement propre si effectués ; installation isolée npm/crates après publication, commandes `--version` et golden CLI.
- **Dépendances/risques :** 4.2 et comptes/certificats du propriétaire ; sans ceux-ci, bloquer la revendication « installation macOS sans avertissement », publier la limite plutôt que simuler la confiance.

### 4.4 — Rendre les artefacts reproductibles et vérifiables

- **Objectif :** relier une version source, des archives et des checksums sans ambiguïté.
- **Changements :** synchroniser versions Cargo/Tauri/npm/changelog ; produire archives et `SHA256SUMS` depuis le tag cible, avec noms stables et notes de release précises ; tester archive CLI, app/DMG et éventuels installateurs depuis un répertoire propre ; enregistrer provenance et résultats par OS. Prévoir un contrôle de concordance tag, version embarquée et checksums avant upload.
- **Parties concernées :** `Cargo.toml`, `apps/desktop/package.json`, `apps/desktop/src-tauri/tauri.conf.json`, `packages/npm/package.json`, `scripts/package*`, `CHANGELOG.md`, `docs/VALIDATION.md`.
- **Acceptation :** chaque asset annoncé existe, se décompresse/s'installe, a le bon `--version`, correspond à `SHA256SUMS` et au tag ; aucun asset non testé sur l'OS annoncé.
- **Validation :** builds propres, `sha256sum`/`shasum`, extraction et smoke du CLI, lancement natif et contrôle de version du paquet final ; revue du contenu des archives.
- **Dépendances/risques :** 4.2–4.3 ; outils de signature et builds multi-OS peuvent rendre les octets non reproductibles, à documenter précisément.

## Phase 5 — Présentation GitHub, contribution et publication (P0)

### 5.1 — Faire du README un essai produit honnête

- **Objectif :** permettre à un visiteur de comprendre, installer et essayer Marklight en moins de quelques minutes.
- **Changements :** ouvrir par le problème et un cas concret, installer par plateforme vérifiée, montrer une sortie terminal **capturée du binaire** et une capture **native** du lecteur ; distinguer CLI npm et app séparée ; lier exemples, limites de dialecte, sécurité, performances, contributions et téléchargements. Remplacer ou étiqueter les badges SVG locaux selon ce qu'ils prouvent réellement. Ajouter texte alternatif et légendes avec provenance des captures.
- **Parties concernées :** `README.md`, `docs/images/`, `docs/images/badges/`, `fixtures/markdown/`, `docs/INSTALLATION.md`.
- **Acceptation :** commandes copiables sur les plateformes citées, images fidèles au binaire publié, liens valides, zéro « disponible » pour un asset absent ; résultat lisible sur GitHub mobile et desktop.
- **Validation :** installation propre et reproduction des captures ; inspection du README rendu et vérification automatisée des liens locaux/externes critiques.
- **Dépendances/risques :** phases 1–4 ; les captures actuelles sont issues du harnais navigateur et doivent rester correctement légendées ou être remplacées par une preuve native.

### 5.2 — Aligner site, documentation et comparaison démontrable

- **Objectif :** éviter que le site, la page npm et le README décrivent des produits ou plateformes différents.
- **Changements :** garder une source de vérité pour versions, plateformes, commandes et limites ; mettre à jour `site/index.html`, `site/docs.html`, `packages/npm/README.md` ; publier une comparaison sobre par cas d'usage uniquement après vérification pratique des alternatives ; ajouter un exemple multi-fichiers téléchargeable et les instructions de retour utilisateur.
- **Parties concernées :** `site/`, `packages/npm/README.md`, `docs/MARKDOWN.md`, `docs/INSTALLATION.md`, `scripts/check-site.mjs`.
- **Acceptation :** chaque lien, exemple et commande correspond à la release réellement disponible ; aucune comparaison non sourcée ni benchmark non comparable ; site utilisable sans JavaScript pour les informations essentielles.
- **Validation :** `node scripts/check-site.mjs`, contrôle des extraits par exécution, inspection responsive/console et test de l'URL publique après déploiement effectif.
- **Dépendances/risques :** 0.1, 4.4 ; les pages publiques peuvent être en retard sur le dépôt et exigent une vérification après publication.

### 5.3 — Rendre l'adoption et les contributions faciles

- **Objectif :** convertir un essai ou un bug reproductible en amélioration utile.
- **Changements :** ajouter modèles courts d'issue/PR pour version, OS, commande, Markdown minimal et résultat attendu, sans demander de document privé ; documenter « good first issue », architecture, commandes de test rapides CLI vs desktop et capture d'UI ; vérifier description, topics, lien du site, catégories et politique de sécurité du dépôt GitHub avant de les afficher.
- **Parties concernées :** `CONTRIBUTING.md`, `SECURITY.md`, `docs/ARCHITECTURE.md`, éventuels `.github/ISSUE_TEMPLATE/` et `.github/PULL_REQUEST_TEMPLATE.md`, métadonnées GitHub.
- **Acceptation :** un nouveau contributeur peut reproduire un bug sur fixture puis lancer la bonne suite ; un nouveau visiteur trouve téléchargement, docs et limites depuis la page du dépôt ; aucune collecte de fichiers sensibles dans les formulaires.
- **Validation :** essai de contribution depuis un checkout propre, rendu des modèles GitHub, revue de la page du dépôt après modification.
- **Dépendances/risques :** 4.1–4.4 ; les métadonnées GitHub sont externes à l'arbre local et doivent être vérifiées, pas déduites de la présence des liens.

### 5.4 — Publier et contrôler la version destinée aux utilisateurs

- **Objectif :** terminer sur une release téléchargeable, installable et dont les promesses correspondent aux essais.
- **Changements :** exécuter les gates, créer le tag et les notes, publier seulement les artefacts validés, vérifier les assets et checksums **depuis GitHub**, tester leur installation, puis mettre à jour README/site/npm et `docs/VALIDATION.md` avec URL, version, OS et preuves. Consigner séparément tout blocage externe (certificat, registre, OS absent).
- **Parties concernées :** `CHANGELOG.md`, `README.md`, `docs/{VALIDATION,INSTALLATION}.md`, `site/`, scripts de release, GitHub Releases et registres effectivement utilisés.
- **Acceptation :** un tiers suit les commandes publiques et obtient le même résultat sur chaque OS annoncé ; tag, source, assets, version interne et documentation concordent ; défauts critiques corrigés avant l'annonce.
- **Validation :** téléchargement neuf, vérification SHA-256, lancement/lecture d'un fixture et fermeture complète, contrôle des liens publics et des logs de release ; les tests locaux seuls ne suffisent pas.
- **Dépendances/risques :** toutes les tâches 0–5 précédentes ; publication, certificats et accès aux comptes du propriétaire. Ne pas annoncer une plateforme bloquée.

## Phase 6 — Vidéo réelle du produit terminé (DERNIÈRE PHASE, P0)

**Condition d'entrée stricte :** commencer uniquement après l'implémentation et la validation de **toutes** les phases 0–5, y compris l'installation du téléchargement final et la mise à jour honnête des pages publiques. La vidéo ne doit pas précéder une correction fonctionnelle ou montrer un autre build que celui publié.

### 6.1 — Capturer un parcours de lecture réel

- **Objectif :** montrer le problème résolu et la valeur du produit final en usage continu.
- **Changements :** appliquer obligatoirement la skill `ffmpeg-video-editor` ; écrire un court scénario basé sur un vrai dossier Markdown et des fichiers non confidentiels ; filmer le téléchargement/installation ou le démarrage du paquet vérifié, la lecture CLI, l'ouverture dans la fenêtre native, le sommaire, la recherche, un lien local, la copie de code et le rechargement après sauvegarde. Enregistrer les sources d'écran originales et la version exacte du binaire.
- **Parties concernées :** nouvel emplacement `docs/demo/` ou `media/`, `README.md` seulement après export ; produit de la release 5.4.
- **Acceptation :** chaque action filmée fonctionne réellement ; problème → installation/démarrage → fonctionnalités principales sont identifiables sans maquette, faux écran, séquence simulée ou promesse hors preuve.
- **Validation :** revoir les prises brutes, répéter les commandes et vérifier que les fichiers ouverts/états affichés proviennent de l'application publiée ; effacer les chemins et données personnelles avant montage.
- **Dépendances/risques :** phase 5.4 terminée ; autorisation d'enregistrer l'écran et disponibilité de l'OS supporté ; captures pouvant révéler des données locales.

### 6.2 — Monter, exporter et vérifier les fichiers finaux

- **Objectif :** fournir une démonstration nette et facile à consulter depuis GitHub.
- **Changements :** avec `ffmpeg-video-editor`, sonder les rushes avec `ffprobe`, monter au rythme des actions réelles, ajouter titres sobres, zooms/recadrages utiles, transitions discrètes et voix/audio nettoyé si nécessaire ; exporter un MP4 lisible sur le Web pour README/GitHub (H.264, `yuv420p`, AAC si audio, `+faststart`) et, si le cadrage s'y prête, une version courte pour réseaux. Ajouter vignette ou lien depuis le README vers l'hébergement vérifié.
- **Parties concernées :** `docs/demo/` ou `media/`, `README.md`, éventuellement `site/` ; fichiers source et exports clairement distingués.
- **Acceptation :** la vidéo finale montre le même produit que 5.4 et reste compréhensible sans musique ; durée, résolution, codecs, poids et lien public documentés ; version courte uniquement si elle conserve une démonstration intelligible.
- **Validation :** `ffprobe` sur chaque export, décodage intégral avec FFmpeg sans erreur, lecture **complète** en lecteur réel avec contrôle visuel/audio, vérification du poids et du lien depuis README/GitHub après upload.
- **Dépendances/risques :** 6.1 ; limites de taille/lecture de l'hébergement choisi, droits éventuels sur musique ou police, lisibilité des détails après compression.

## Résultat attendu

Après exécution, Marklight doit pouvoir être essayé immédiatement depuis une page GitHub claire, sur chaque plateforme réellement validée, avec un parcours de lecture fluide, une sécurité argumentée par tests, des artefacts installables et traçables, des limites explicites, et une démo filmée sur la version distribuée. La qualité et la facilité de partage augmenteront les chances d'adoption ; les étoiles resteront une conséquence possible, jamais un critère de validation technique.
