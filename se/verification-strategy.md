# Verification strategy

Verification asks whether each lower-level artifact correctly realizes its
governing information. Evidence grows as the project gains formal models and
implementations.

For every change:

1. inspect traceability from affected design statements to requirements;
2. check formal rules for coverage, internal consistency, and cross-domain
   compatibility;
3. derive positive, negative, boundary, and interaction cases from rule IDs;
4. run applicable tests against every implementing tool; and
5. compare interpreter and compiled behavior where both exist.

Mermaid diagrams are informative verification aids, not formal evidence.
Claims of consistency, totality, race freedom, or deadlock freedom require an
explicit argument, executable model, proof, or conservative checker appropriate
to the claim. Assumptions and verification limits shall be recorded.
