#set page(
    numbering: "1"
)

#set align(left)
#set text(
    lang: "en",
    size: 13pt
)

#set par(
    first-line-indent: (
        amount: 1.5em,
        all: false,
    ),
    spacing: 0.65em,
    justify: true,
)

#show link: underline

#let alinea = [#box(height: 1em, width: 1.7em)[]]

#let jump = (x) => [#line(start: (0pt, x*1pt), length: 0%)]

#let labels = (
    <s1>,
    <s2>,
    <s3>,
    <s4>,
    <s5>
)

#let sections = (
    "Introduction",
    "Transformation du modèle Relationnel au modèle Graphe",
    "Migration d’une BDD PostgreSQL vers Neo4j",
    "Génération de requêtes Cypher",
    "Implémentation"
)

// Table content :
#align(center)[= IN608 : _Neo4j-Migrator_]
#jump(15)

#align(center)[== -- _Loïc Desmarès_ --]
#jump(55)

= Sommaire
#jump(15)

#for index in range(sections.len()) {
    let number = index+1
    text(
        size: 16pt,
        [   
            *#number* - #link(labels.at(index))[#sections.at(index)]
            #jump(5)
        ]
    )
}

#jump(150)
== Note :
#alinea Ce rapport explique les tenants et aboutissants du projet. Il comporte quelques courts exemples de code _*Cypher*_ générés, mais la majorité du code _*Rust*_
est trouvable sur le dépôt Github :
#link("https://github.com/LugolBis/Neo4j-Migrator#")[Repository-Github-Neo4j-Migrator].

// Introduction
#pagebreak()
== 1 - #sections.at(0) <s1>
#jump(2)
#alinea L'enjeu de ce projet est de réaliser la transformation d'un modèle de base de données relationnel à un modèle de base de données graphe. 
Afin de réaliser, à terme, la migration d'une base de données relationnelle vers une base de données graphe.
#jump(10)

// Transformation
== 2 - #sections.at(1) <s2>
#jump(2)
=== Quels sont les concepts principaux du modèle _Relationnel_ ?
#jump(5)
- Les *Tables* : structurent et traduisent des *concepts*.

- Les *Clés primaires* : traduisent l'*unicité* des objets stockés.

- Les *Clés étrangères* : *relient* les données entre elles.

- Les *Contraintes de type* : assurent la *cohérence* des données.
#jump(5)

=== Traduction des concepts :
#jump(5)
#table(
    columns: 2,
    column-gutter: 5%,
    align: center,
    inset: 10pt,
    table.header(
        [*Modèle _Relationnel_*], [*Modèle _Graphe_*]
    ),
    [Tables], [Étiquettes],
    [Lignes], [Noeuds],
    [Clés primaires], [Propriétés uniques],
    [Clés étrangères], [Relations/Arcs],
    [Contraintes de type], [Contraintes de type]
)
#jump(5)

=== Problème de sémantique :
#jump(1)
#alinea Comment garder la cohérence sémantique en passant d'un modèle à l'autre ? Il est important de conserver le sens sémantique des données stockées pour pouvoir facilement
les utiliser même après la transformation au modèle graphe.
#jump(5)

- Les *Étiquettes* : Chaque noeud du graphe en possède une à plusieurs.
    Ainsi chaque noeud a comme étiquette le nom de la *table* dont il est issu.
    #jump(4)
- Les *Relations* : Leur nom est formé à partir de l'*étiquette* du noeud de départ
    et de la *colonne* de la *table* courante référencée.\
    #block(
        fill: luma(230),
        inset: 8pt,
        radius: 4pt,
        [
            Exemple -- Les clés étrangères sont #underline[soulignées] -- \
            Modèle _Relationnel_ : Commande (*id*, #underline[user], price)\
            Nom de la *Relation* : COMMANDE\_ref\_USER
        ]
    )
#jump(5)

=== Contraintes d'intégrité :
Comment garder la cohérence des données dans un graphe ?
#jump(5)

- Les *Propriétés uniques* : Chaque noeud peut disposer d'une *propriété* dont la
    *valeur* est *unique*. (Son implémentation est native à _Cypher_)
    #jump(4)
- Les *Contraintes de type* : Chaque *propriété* d'un noeud a un *type* donné.\
    (Son implémentation n'est pas native à _Cypher_)
#jump(10)

// Migration
== 3 - #sections.at(2) <s3>
#jump(2)
=== 3.1 - Export massif depuis _PostgreSQL_ :
#jump(2)
#alinea On exporte les métadonnées et les tables de la base de données _PostgreSQL_ à l'aide de l'outil _pgsql_, respectivement au format JSON et CSV.
#jump(5)

=== 3.2 - Transformation des données :
#jump(2)
+ Génération des en-têtes des CSV à partir des métadonnées :
    #jump(1)
    - On génère les en-têtes des CSV des Noeuds :\
        Exemple :
        #block(
            fill: luma(230),
            inset: 8pt,
            radius: 4pt,
            [:ID ; property1 : STRING ; :LABEL]
        )
    - On génère les en-têtes des CSV des Relations :\
        Exemple :
        #block(
            fill: luma(230),
            inset: 8pt,
            radius: 4pt,
            [:START_ID ; :END_ID ; :TYPE]
        )
    #jump(1)
    #alinea On distingue donc les *colonnes* qui sont des *clés étrangères* (*Relation*) de celles qui ne le sont pas.
    #jump(70)
+ Génération des contraintes d'intégrité à partir des métadonnées :
    #jump(1)
    - On génère les contraintes d'*unicité* et *not null*.\
        Exemple :
        #block(
            fill: luma(230),
            inset: 8pt,
            radius: 4pt,
            [
                create constraint UO if not exists for (n:ORDERS) require n.NAME is unique;\
                create constraint UNN if not exists for (n:ORDERS) require n.NAME is not null;
            ]
        )
        #jump(2)
    - On génère des triggers *APOC* pour les *types* (on déduit le type _Cypher_ à partir du type _PostgreSQL_).\
            Exemple :
            #block(
                fill: luma(230),
                inset: 8pt,
                radius: 4pt,
                [
                    CALL apoc.trigger.add('TID', "MATCH (m:ORDERS) WHERE m.NAME IS NOT NULL AND NOT valueType(m.NAME) = 'STRING' 
                    CALL apoc.util.validate(true, 'ERROR', []) RETURN m", {phase: 'before'});
                ]
            )
            #jump(2)
+ Extraction et formatage des *Noeuds* :\
    #jump(1)
    #alinea On lit le dossier contenant les tables au format CSV, puis on charge celles-ci dans des *DataFrame*.
    On utilise les *DataFrame* pour sélectionner seulement les colonnes présentes dans les en-têtes.\
    #alinea Puis on génère et insère une nouvelle colonne id contenant les _*ID*_ générés (_LABEL_ + _ROWNUM_), le _ROWNUM_ étant le numéro de la ligne.
    Enfin on génère et insère une dernière colonne contenant le _*LABEL*_ (_Étiquette_).
    Au terme de ces opérations on écrit le contenu des *DataFrame* dans les fichiers CSV d'import correspondants.
    #jump(2)
+ Extraction et formatage des *Relations* :\
    #jump(1)
    #alinea On lit le fichier contenant les tuples : *table*, *colonne référençant*,*table référencée*, *colonne référencée* (précédemment généré),
    à partir de celui-ci on charge dans deux *DataFrame* distincts les deux tables du tuple.\
    #alinea Puis on fait une jointure entre les deux *DataFrame* et on sélectionne uniquement la colonne contenant les _*ID*_ générés.
    Enfin on génère et insère une dernière colonne contenant le "_*Label*_" de la *relation*.
    De même que pour les *Noeuds* on écrit le contenu des *DataFrame* dans les fichiers CSV d'import correspondants.
#jump(5)

=== 3.3 - Chargement massif vers _Neo4j_ :
#jump(2)
#alinea On construit la commande *shell* permettant de réaliser le chargement massif des données
à partir du dossier d'import de la base de données _Neo4j_ cible.
On utilise ici l'outil de chargement massif : *```shell neo4j-admin```*.

// Traduction
== #4 - #sections.at(3) <s4>
#jump(2)
#alinea On construit l'*AST* d'une requête _SQL_ à l'aide de _sql_parser_, puis on en extrait les données (colonnes sélectionnées, alias des tables, etc).
On traduit les concepts du _SQL_ ainsi :\
- La clause *from* est traduite par la clause *match* pour les tables simplements sélectionnées et pour les jointure simples. Les jointures plus complexes sont traduites à l'aide
    de la clause *match optionnal*.
- La clause *select* est traduite par la clause *return*.
#jump(10)

// Rust
== #5 - #sections.at(4) <s5>
#jump(2)
#alinea _*Rust*_ a été retenu pour ses très bonnes performances comparées à des langages comme _*Python*_. En effet le _*Rust*_ offre de bien 
meilleures performances, ce qui fut un choix déterminant pour le développement d'un projet voué à traiter de gros volumes de données.\
#alinea De plus dans le cadre de transformation de données ne nécessitant pas la manipulation de structures de données trop complexes (qui pourraient être plus complexe
à manipuler à cause des notions d'emprunt), _*Rust*_ s'impose comme une solution naturelle et simple d'utilisation pour manipuler des fichiers.
Enfin il est aisé d'intégrer des crates (module) très puissant tel que _*Polars*_, qui est une crate développé en _*Rust*_ permettant de manipuler simplement des _*DataFrame*_,
dont les opérations sont optimisées et parallélisées.
#jump(2)
#alinea Les connexions aux bases de données _*PostgreSQL*_ et _*Neo4j*_ sont implémentées avec deux structures permettant d'interfacer avec celles-ci via leur CLI respective.
Elles mettent donc en place une abstraction sans coût pour communiquer facilement avec les bases de données.
Une autre crate nommée _*serde_json*_ est utilisée pour interpréter et manipuler les métadonnées stockées dans un fichier JSON.