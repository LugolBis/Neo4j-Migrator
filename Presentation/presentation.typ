#import table: cell, header

#set page(
    flipped: true,
    numbering: "1",
    fill: gradient.linear(rgb(87,156,184), rgb(227,149,71), angle: 45deg)
)

#set align(left)
#set text(
    size: 20pt
)

= Neo4j-Migrator
==
===
== - Transformation du modèle _Relationnel_ au modèle _Graphe_
==
== - Migration d'une BDD _PostgreSQL_ vers _Neo4j_
== 
== - Génération de requêtes _Cypher_

#pagebreak()

== Neo4j-Migrator : _Relationnel_ $->$ _Graphe_
==
==
== Quels sont les concepts principaux du model _Relationnel_ ?
===
- Les *Tables* $->$ structurent et traduisent des *concepts*

- Les *Clés primaires* $->$ traduisent l'*unicité* des objets stockés

- Les *Clés étrangères* $->$ *relient* les données entre elles

- Les *Contraintes de type* $->$ assurent la *cohérence* des données

#pagebreak()

== Neo4j-Migrator : _Relationnel_ $->$ _Graphe_
#line(start:(0pt, 25pt), length: 0%)

== Traduction des concepts :
#line(start:(0pt, 15pt), length: 0%)
#table(
    columns: 2,
    column-gutter: 5%,
    fill: rgb(181, 143, 115),
    align: center,
    inset: 10pt,
    table.header(
        [*Modèle _Relationnel_*], [*Modèle _Graphe_*]
    ),
    [Tables], [Labels],
    [Lignes], [Noeuds],
    [Clés primaires], [Propriétés uniques],
    [Clés étrangères], [Relations/Arcs],
    [Contraintes de type], [Contraintes de type]
)

#pagebreak()
== Neo4j-Migrator : _Relationnel_ $->$ _Graphe_
#line(start:(0pt, 25pt), length: 0%)

== Problème de sémantique :
Comment garder la cohérence sémantique en passant d'un modèle à l'autre ?
#line(start:(0pt, 2pt), length: 0%)

+ Les *Labels* : Chaque noeud du graphe en possède un à plusieurs.
    Ainsi chaque noeud a comme label le nom de la *table* dont il est issu.
    #line(start:(0pt, 4pt), length: 0%)
+ Les *Relations* : Leur nom est formé à partir du *label* du noeud de départ
    et de la *colonne* de la *table* courante référencée.

    Exemple :
    #block(
        fill: rgb(181, 143, 115),
        inset: 8pt,
        radius: 4pt,
        [
            Modèle _Relationnel_ : Commande (*id*, #underline[user], price)

            Nom de la *Relation* : COMMANDE\_\_REF\_\_USER
        ]
    )

#pagebreak()
== Neo4j-Migrator : _Relationnel_ $->$ _Graphe_
#line(start:(0pt, 25pt), length: 0%)

== Contraintes d'intégritées :
Comment garder la cohérence d'un graphe ?
#line(start:(0pt, 2pt), length: 0%)

+ Les *Propriétés uniques* : Chaque noeud peut disposer d'une *propriété* dont la
    *valeur* est *unique*. (Son implémentation est native à _Cypher_)
    #line(start:(0pt, 4pt), length: 0%)
+ Les *Contraintes de type* : Chaque *propriété* d'un noeud a un *type* donné.\
    (Son implémentation n'est pas native à _Cypher_)

#pagebreak()
== Neo4j-Migrator : _PostgreSQL_ $->$ _Neo4j_
#line(start:(0pt, 25pt), length: 0%)

== 1 - Export massif depuis _PostgreSQL_ :
#line(start:(0pt, 2pt), length: 0%)

- Utilisation de _pgsql_ et de ```bash \copy```
    #line(start:(0pt, 4pt), length: 0%)
- Export des méta données au format JSON avec une requête SQL (via _pgsql_).
    #line(start:(0pt, 4pt), length: 0%)
- Export des tables de la base de données au format CSV.
    #line(start:(0pt, 4pt), length: 0%)

#pagebreak()
== Neo4j-Migrator : _PostgreSQL_ $->$ _Neo4j_
#line(start:(0pt, 25pt), length: 0%)

== 2 - Transformation des données :
#line(start:(0pt, 2pt), length: 0%)

=== 2.1 : Génération des Headers des CSV & Génération des contraintes d'intégritées
    #line(start:(0pt, 4pt), length: 0%)
=== 2.2 : Extraction et formatage des Noeuds
    #line(start:(0pt, 4pt), length: 0%)
=== 2.3 : Extraction et formatage des Relations
    #line(start:(0pt, 4pt), length: 0%)

#pagebreak()
== Neo4j-Migrator : _PostgreSQL_ $->$ _Neo4j_
#line(start:(0pt, 25pt), length: 0%)

== 2.1 - Génération des Headers des CSV & Génération des contraintes d'intégritées
#line(start:(0pt, 2pt), length: 0%)

À partir des méta données :
- On génère les Headers des CSV des Noeuds :
    #block(
        fill: rgb(181, 143, 115),
        inset: 8pt,
        radius: 4pt,
        [:ID ; property1 : STRING ; :LABEL]
    )
- On génère les Headers des CSV des Relations :
    #block(
        fill: rgb(181, 143, 115),
        inset: 8pt,
        radius: 4pt,
        [:START_ID ; :END_ID ; :TYPE]
    )

On distingue donc les *colonnes* qui sont des *clés étrangères* (*Relation*) de celles qui ne le sont pas.

#pagebreak()
== Neo4j-Migrator : _PostgreSQL_ $->$ _Neo4j_
#line(start:(0pt, 25pt), length: 0%)

== 2.1 - Génération des Headers des CSV & Génération des contraintes d'intégritées
#line(start:(0pt, 2pt), length: 0%)

Toujours à partir des méta données :
- On génère les contraintes d'*unicitée* et *not null* : \
    #block(
        fill: rgb(181, 143, 115),
        inset: 8pt,
        radius: 4pt,
        [create constraint UO if not exists for (n:ORDERS) require n.NAME is unique;
        create constraint UNN if not exists for (n:ORDERS) require n.NAME is not null;]
    )
- On génère des triggers *APOC* pour les *types* :
    #block(
        fill: rgb(181, 143, 115),
        inset: 8pt,
        radius: 4pt,
        [CALL apoc.trigger.add('TID', "MATCH (m:ORDERS) WHERE m.NAME IS NOT NULL AND NOT valueType(m.NAME) = 'STRING' CALL apoc.util.validate(true, 'ERROR', []) RETURN m", {phase: 'before'});]
    )