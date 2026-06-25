# Discrete mathematics

1.     Book: Kenneth H. Rosen. 2021. *Discrete Mathematics and Its Applications*. 8th Edition. McGraw Hill: McGraw Hill Education (India) Private Limited.

- Proposition - a declarative sentence that is either true or false , but not both. true is demotes by $T$ , false is denoted by $F$ and proposition variables are denoted by $p,q,r...$ or $p_1,p_2,p_3...$
- Compound propositions are new propositions created from excising proposition using logical operators or connectives.

Connectives

- Negation of proposition is denoted by $\neg p$, the truth value of the negation of $p$ is the opposite of truth value of $p$.
- conjunction - lets take 2 proposition say p and q , the conjunction ($\land$) $p \land q$ is *“p and q”* as in logical and. it is true if both p and q are true
- Disjunction ($\lor$) - it is like a logical or, it is false when both p and q are false otherwise true.
- implication or conditional statement, $p \implies q$  (p implies q or p conditional q) is the proposition *“if p, then q” . $p \implies q$* is false when p is true and q is false. here p is called hypothesis and q is called conclusion.
- converse , $q \implies p$ is the converse of $p \implies q$
- contrapositive , $\neg q \implies \neg p$ is the contrapositive of $p \implies q$

### Logical equivalences

a compount proposition that is always true, ie. true for every combination of truth values of its propositional variables is called tautology.

for example take $p \lor \neg p$ , this will always evaluate to true , so it is a tautology .

the opposit of tatutolody , ie always false is called contradiction
for example , $p \land \neg p$ will always evaluate to false , which is a contradiction

compount proposition p and q are called **logicaly equivalent** if their truth table are the same.it is denoted by $ p \equiv q $
example : $ p \implies q $ and $ \neg p \implies \neg q $

DeMorgans laws :
- $\neg(p \land q) \equiv \neg p \lor \neg q$
- $\neg(p \lor q) \equiv \neg p \land \neg q$
- $ p \implies q \equiv \neg p \lor q$

how to state two compount propositons are not logically equaivalent ?
- find any one row in the truth table where one evaluates to true and other evaluates to false --> this consitions are called a counterexample


### proof methodology

a proof is a valid argument to establish the truth of a mathematical statement.
it can use
- the hypotheses of the theorem
- axioms or defenitions assumed to be true
- previously proven theorms
and should follow rule of inference

a proof is said to be valid if the trut of the statement is being proved

Terminology
- theorem - a mathamatical statement that can be shown to be true
- propositon and lemmas - minor theorems used as intermediaries to prove large theorems
- corollary - a theorem that can be established directly from a proven theorem
- conjecture - a statement that is being proposed to be a true statement , usually on the basis of some partial evidence

### Direct Proof example

we seek to solve a $p \implies q$ problem using direct proof technique

**Q** : prove theorem , if $n$ is an odd integer , prove $n^2$ is also and odd integer
**Sol** :

This is a $p \implies q$ problem ,
Suppose $n$ is an odd integer , means by defenition $ n = 2k+1$ for some integer k
so ,
$$
\displaylines{
n^2 = (2k +1)^2 \\ n^2 = 4k^2 + 4k +1 \\ n^2 = 2(2k^2+2k)+1}
$$

since k is an integer, $2k^2+2k =l$ is also an integer
so,
$$
n^2 = 2l+1
$$
that is $n^2$ is an odd integer

### Indirect proof example (proof by contrapositon)


we seek to solve a $p \implies q$ problem using indirect proof technique, but unlike direct proof we will use the propert,
$$
(p \implies q) \equiv (\neg p \implies \neg q)
$$

**Q** : prove theorem , if $3n+2$ is odd , where $n$ is an integer , then $n$ is odd
**Sol** :
Proof by contrapositon
the contrapositive of the theorem statement is , if $3n+2$ is not odd ,
$n$ is an integer , then $n$ is not odd
ie, if $n$ is an even integer then $3n+2$ is even
ie,
$$
n=2k
$$
for some integer k
now ,
$$
\displaylines{
3n + 2 = 3(2k) + 2 \\
3n+2 = 6k+2 \\
3n+2 = 2(3k + 1)}
$$
since $k$ is an integer , $3k+1 =l$ is also an intefer
ie,
$$
3n+2 = 2l
$$
thus by defenition of even number the theorem is proved

### Proof by contradiction

contradiction - it is a proposition that is always false

here we seek to solve the problem of kind $p$ by using $\neg p \implies F$

#### Example
**Q** : prove the theorem, $\sqrt{2}$ is irrational
**Sol** :
By way of contradiction ,suppose $\sqrt{2}$ is rational,
by defenition of rational number , $\sqrt{2} = {\frac{p}{q}}$ where $p$ and $q$ are integers and $q \neq 0$

Further suppose $p,q$ has no common factors,
in particular , $p,q$ are not both even numbers
squaring,
$$
\displaylines{
2 = (\frac{p}{q})^2 \\
p^2 = 2q^2
}
$$

because $p$ are integers and , $p^2$ are even , $p$ is even
that is , $p=2k$ where $k$ is an integer
that is , $4k^2 = 2q^2$
that is , $q^2 = 2k^2$

so by defenition, $q$ is even
we assumed that both $p,q$ are not even , therefor we have a contradiction,
so we can conclude the initial statement to be true
