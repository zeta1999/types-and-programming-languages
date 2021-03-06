# System Fω

This is an implementation (mostly of just the type system) of the higher-order polymorphic lambda calculus with explicit typing. This allows us to express functions with impredicative arguments, and we can emulate Haskell style typeclasses using functors over existential types (akin to 1ML) - something not expressible in Standard ML. 


#### References
I've included some selected references on implementing System Fw, particularly with respect to the addition of recursive types (System F omega-mu):

-  Mendler, N.P.: Recursive types and type constraints in second-order lambda calculus. In: Proceedings of the Second Annual IEEE Symposium on Logic in Computer
Science, Ithaca, N.Y., IEEE Computer Society Press (1987) 30–36

   - One of the classic, highly reference papers


- Andreas Abel, Ralph Matthes, Tarmo Uustalu, "Iteration and coiteration schemes for higher-order and nested datatypes", Theoretical Computer Science, Volume 333, Issues 1–2, 2005, Pages 3-66,

   - Interesting implementations, a paper that allowed me to better understand "Mendler iterations"

- Andreas Abel, Ralph Matthes, "Fixed Points of Type Constructors and Primitive Recursion", International Workshop on Computer Science Logic (2004)

- Ahn, Ki Yung, "The Nax Language: Unifying Functional Programming and Logical Reasoning in a Language based on Mendler-style
Recursion Schemes and Term-indexed Types" (2014). Dissertations and Theses. Paper 2088.

   - This reference is quite useful as it restates much of the literature in easily understandable terms

- Yufei Cai, Paolo G. Giarrusso, and Klaus Ostermann. 2016. System f-omega with equirecursive types for datatype-generic programming. SIGPLAN Not. 51, 1 (January 2016), 30-43. DOI: https://doi.org/10.1145/2914770.2837660

   - Interesting, but I found it to be of not too much practical use

- A Polymorphic Lambda-Calculus with Sized Higher-Order Types, Andreas Abel, PhD thesis

   -  Page 76 seems to give some good hints on how to type Fold/Unfold operators (but for equirecursive, isorecursive on pp 85). It also seems that the kinding rules/type-equivalence can treat a type constructor abstraction (* => *) and a recursive type of kind (* => *) as the same. Iso-coinductive construtors are also discussed, pp 157.
