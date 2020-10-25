# Context-free grammar

Nonterminals are highlighted

`program` -> def main { `decls` `stmts` }

## Declarations

`decls` -> let `identifier`: `type`;

`identifier` -> `latin_letters` `digits`

`latin_letters` -> `latin_letters` `latin_letter` | `latin_letter`

`latin_letter` -> a | b | ... | z

`digits` -> `digits` `digit` | `digit`

`digits` -> `digit`

`digit` -> 0 | 1 | ... | 9

## Statements

`stmts` -> `stmts` `stmt` | `stmt` | ε