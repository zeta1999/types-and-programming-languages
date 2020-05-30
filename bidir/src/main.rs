//! "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism"
//! Paper by J. Dunfield and N. Krishnaswami
//!
//! Also see very useful Haskell implementation:
//! https://github.com/lexi-lambda/higher-rank/

use std::collections::HashSet;

/// A source-level type
#[derive(Clone, Debug, PartialEq)]
enum Type {
    Unit,
    Var(usize),
    Arrow(Box<Type>, Box<Type>),
    Exist(usize),
    Univ(Box<Type>),
}

impl Type {
    fn monotype(&self) -> bool {
        match &self {
            Type::Univ(_) => false,
            Type::Arrow(t1, t2) => t1.monotype() && t2.monotype(),
            _ => true,
        }
    }

    /// Collect the free existential variables of the type
    fn freevars(&self) -> Vec<usize> {
        fn walk(ty: &Type, vec: &mut Vec<usize>) {
            match ty {
                Type::Unit | Type::Var(_) => {}
                Type::Exist(v) => vec.push(*v),
                Type::Arrow(a, b) => {
                    walk(a, vec);
                    walk(b, vec);
                }
                Type::Univ(a) => walk(a, vec),
            }
        }
        let mut v = Vec::new();
        walk(self, &mut v);
        v
    }

    /// Perform subsitution of type `s` into self, algorithm from TAPL
    fn subst(&mut self, s: &Type) {
        fn walk<F: Fn(&mut Type)>(t: &mut Type, c: usize, f: &F) {
            match t {
                Type::Var(n) if *n == c => f(t),
                Type::Arrow(a, b) => {
                    walk(a, c, f);
                    walk(b, c, f);
                }
                Type::Univ(a) => walk(a, c + 1, f),
                _ => {}
            }
        }
        walk(self, 0, &|f| *f = s.clone());
    }
}

/// An expression in our simply typed lambda calculus
#[derive(Clone, Debug, PartialEq)]
enum Expr {
    /// The unit expression, ()
    Unit,
    /// A term variable, given in de Bruijn notation
    Var(usize),
    /// A lambda abstraction, with it's body. (\x. body)
    Abs(Box<Expr>),
    /// Application (e1 e2)
    App(Box<Expr>, Box<Expr>),
    /// Explicit type annotation of a term, (x : A)
    Ann(Box<Expr>, Box<Type>),
}

/// An element in the typing context
#[derive(Clone, Debug, PartialEq)]
enum Element {
    /// Universal type variable
    Var,
    /// Term variable typing x : A. We differ from the paper in that we use
    /// de Bruijn indices for variables, so we don't need to mark which var
    /// this annotation belongs to - it always belongs to the innermost binding (idx 0)
    /// and we will find this by traversing the stack
    Ann(Type),
    /// Unsolved existential type variable
    Exist(usize),
    /// Existential type variable that has been solved
    /// to some monotype
    Solved(usize, Type),
    /// I am actually unsure if we really need a marker, due to how we structure
    /// scoping, see `with_scope` method.
    Marker(usize),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    /// We model the algorithmic context as a simple stack of elements
    ctx: Vec<Element>,
    /// We assign fresh exist. variables a unique, strictly increasing number
    ev: usize,
}

impl Context {
    /// Generate a fresh identifier
    fn fresh_ev(&mut self) -> usize {
        let e = self.ev;
        self.ev += 1;
        e
    }

    /// Requires a mutable reference to self because we need to push/pop onto the stack
    /// in the case of universally quantified variables. However, this can be considered
    /// mostly immutable, since self should be equal before and after the call
    fn well_formed(&mut self, ty: &Type) -> bool {
        match ty {
            Type::Exist(alpha) => self.ctx.contains(&Element::Exist(*alpha)) || self.find_solved(*alpha).is_some(),
            Type::Univ(alpha) => self.with_scope(Element::Var, |f| f.well_formed(&alpha)),
            Type::Var(idx) => self.find_type_var(*idx),
            Type::Arrow(a, b) => self.well_formed(&a) && self.well_formed(&b),
            Type::Unit => true,
        }
    }

    fn check_wf(&mut self, ty: &Type) -> Result<bool, String> {
        if self.well_formed(ty) {
            Ok(true)
        } else {
            dbg!(&self.ctx);
            Err(format!("Type {:?} is not well formed!", ty))
        }
    }

    // Pop off any stack growth incurred from calling `f`
    fn with_scope<T, F: Fn(&mut Context) -> T>(&mut self, e: Element, f: F) -> T {
        println!("\u{001b}[31msaving scope @ {:?} \u{001b}[0m", e);
        self.ctx.push(e.clone());
        let t = f(self);

        while self.ctx[self.ctx.len() - 1] != e {
            println!("\u{001b}[31mpopping scope @ {:?} \u{001b}[0m", self.ctx.pop());
        }
        self.ctx.pop();
        t
    }

    /// Apply the context to a type, replacing any solved existential variables
    /// in the context onto the type, if it contains a matching existential
    fn apply(&self, ty: Type) -> Type {
        match ty {
            Type::Unit | Type::Var(_) => ty,
            Type::Arrow(a, b) => Type::Arrow(Box::new(self.apply(*a)), Box::new(self.apply(*b))),
            Type::Univ(ty) => Type::Univ(Box::new(self.apply(*ty))),
            Type::Exist(n) => {
                match self.find_solved(n) {
                    // Apply to the solved variable also - this is important
                    // since we can have solved references deeper in the stack
                    Some(solved) => self.apply(solved.clone()),
                    None => ty,
                }
            }
        }
    }

    /// Find the term annotation corresponding to de Bruijn index `idx`.
    /// We traverse the stack in a reversed order, counting each annotation
    /// we come across
    fn find_annotation(&self, idx: usize) -> Option<&Type> {
        let mut ix = 0;
        for elem in self.ctx.iter().rev() {
            match &elem {
                Element::Ann(ty) => {
                    if ix == idx {
                        return Some(&ty);
                    }
                    ix += 1
                }
                _ => {}
            }
        }

        None
    }

    /// Find the term annotation corresponding to de Bruijn index `idx`.
    /// We traverse the stack in a reversed order, counting each annotation
    /// we come across
    fn find_type_var(&self, idx: usize) -> bool {
        let mut ix = 0;
        for elem in self.ctx.iter().rev() {
            match &elem {
                Element::Var => {
                    if ix == idx {
                        return true;
                    }
                    ix += 1
                }
                _ => {}
            }
        }
        false
    }

    /// Find the monotype associated with a solved existential variable `alpha`
    /// in the context, if it exists.
    fn find_solved(&self, alpha: usize) -> Option<&Type> {
        for elem in &self.ctx {
            match &elem {
                Element::Solved(n, ty) if *n == alpha => return Some(ty),
                _ => {}
            }
        }
        None
    }

    /// This is one of the more confusing parts of the paper, IMO. We have to open
    /// a 'hole' in the context, where we can replace/insert some arbitrary amount
    /// of bindings where an unsolved existential (or marker, in the paper) was
    /// previously located
    fn splice_hole<F: Fn(&mut Vec<Element>)>(&mut self, exist: usize, f: F) -> Result<(), String> {
        let mut ret = None;
        for (idx, el) in self.ctx.iter().enumerate() {
            match el {
                Element::Exist(n) if *n == exist => ret = Some(idx),
                _ => {}
            }
        }
        let idx = ret.ok_or_else(|| format!("{} not bound in ctx", exist))?;
        // println!("splicing {} {}", exist);
        let rest = self.ctx.split_off(idx + 1);
        self.ctx.pop();
        println!("splicing {}, {:?}, {:?}", exist, self.ctx, rest);
        f(&mut self.ctx);
        self.ctx.extend(rest);
        Ok(())
    }

    fn split_context(&mut self, exist: usize) -> Result<(&mut Self, Vec<Element>), String> {
        let mut ret = None;
        for (idx, el) in self.ctx.iter().enumerate() {
            match el {
                Element::Exist(n) if *n == exist => ret = Some(idx),
                _ => {}
            }
        }
        let idx = ret.ok_or_else(|| format!("{} not bound in ctx", exist))?;
        let rest = self.ctx.split_off(idx + 1);
        self.ctx.pop();
        Ok((self, rest))
    }

    fn subtype(&mut self, a: &Type, b: &Type) -> Result<(), String> {
        // println!("{:?}", self.ctx);

        use Type::*;
        match (a, b) {
            // Rule <: Unit
            (Unit, Unit) => Ok(()),
            // Rule <: Var
            (Var(a), Var(b)) if a == b => Ok(()),
            // Rule <: Exvar
            (Exist(a), Exist(b)) if a == b => Ok(()),
            // Rule <: ->
            (Arrow(a1, a2), Arrow(b1, b2)) => {
                self.subtype(b1, a1)?;
                self.subtype(&self.apply(*a2.clone()), &self.apply(*b2.clone()))
            }
            // Rule <: forall. L
            (Univ(a), b) => {
                let alpha = self.fresh_ev();
                let mut a_ = *a.clone();
                a_.subst(b);
                self.with_scope(Element::Marker(alpha), |f| {
                    f.ctx.push(Element::Exist(alpha));
                    f.subtype(&a_, b)
                })
            }
            // Rule <: forall. R
            (a, Univ(b)) => {
                let alpha = self.fresh_ev();
                self.with_scope(Element::Var, |f| f.subtype(a, b))
            }
            // Rule <: InstantiateL
            (Exist(alpha), a) if !a.freevars().contains(alpha) => self.instantiateL(*alpha, a),
            // Rule <: InstantiateR
            (a, Exist(alpha)) if !a.freevars().contains(alpha) => self.instantiateR(a, *alpha),
            (a, b) => Err(format!("{:?} is not a subtype of {:?}", a, b)),
        }
    }

    fn instantiateL(&mut self, alpha: usize, a: &Type) -> Result<(), String> {
        println!("InstL Exist({}) <: {:?}", alpha, a);
        let (l, r) = self.split_context(alpha)?;
        // InstLSolve
        if a.monotype() && l.well_formed(a) {
            println!("{:?}", l.ctx);
            l.ctx.push(Element::Solved(alpha, a.clone()));
            l.ctx.extend(r);
            return Ok(());
        // self.splice_hole(alpha, |ctx| ctx.push(Element::Solved(alpha, a.clone())))
        } else {
            l.ctx.push(Element::Exist(alpha));
            l.ctx.extend(r);
        }
        match a {
            // InstLArr
            Type::Arrow(A1, A2) => {
                let a1 = self.fresh_ev();
                let a2 = self.fresh_ev();

                self.splice_hole(alpha, |ctx| {
                    ctx.push(Element::Exist(a2));
                    ctx.push(Element::Exist(a1));
                    ctx.push(Element::Solved(
                        alpha,
                        Type::Arrow(Box::new(Type::Exist(a1)), Box::new(Type::Exist(a2))),
                    ));
                })?;
                self.instantiateR(A1, a1)?;
                let A2_ = self.apply(*A2.clone());
                self.instantiateL(a2, &A2_)
            }
            // InstLAllR
            Type::Univ(beta) => unimplemented!(),
            // InstLReach
            Type::Exist(beta) => {
                println!("InstLReach {:?}", l);
                self.splice_hole(*beta, |ctx| ctx.push(Element::Solved(*beta, Type::Exist(alpha))))
            }
            _ => Err(format!("Could not instantiate Exist({}) to {:?}", alpha, a)),
        }
    }

    fn instantiateR(&mut self, a: &Type, alpha: usize) -> Result<(), String> {
        println!("InstR {:?} <: Exist({}) ", a, alpha);
        // InstRSolve
        let (l, r) = self.split_context(alpha)?;
        // InstLSolve
        if a.monotype() && l.well_formed(a) {
            println!("{:?}", l.ctx);
            l.ctx.push(Element::Solved(alpha, a.clone()));
            l.ctx.extend(r);
            return Ok(());
        // self.splice_hole(alpha, |ctx| ctx.push(Element::Solved(alpha, a.clone())))
        } else {
            l.ctx.push(Element::Exist(alpha));
            l.ctx.extend(r);
        }
        match a {
            // InstRArr
            Type::Arrow(A1, A2) => {
                let a1 = self.fresh_ev();
                let a2 = self.fresh_ev();

                self.splice_hole(alpha, |ctx| {
                    ctx.push(Element::Exist(a2));
                    ctx.push(Element::Exist(a1));
                    ctx.push(Element::Solved(
                        alpha,
                        Type::Arrow(Box::new(Type::Exist(a1)), Box::new(Type::Exist(a2))),
                    ));
                })?;
                self.instantiateL(a1, &A1)?;
                let A2_ = self.apply(*A2.clone());
                self.instantiateR(&A2_, a2)
                // unimplemented!()
            }
            // InstalRAllR
            Type::Univ(beta) => unimplemented!(),
            // InstallRReach
            Type::Exist(beta) => self.splice_hole(*beta, |ctx| ctx.push(Element::Solved(*beta, Type::Exist(alpha)))),
            _ => Err(format!("Could not instantiate Exist({}) to {:?}", alpha, a)),
        }
    }

    fn infer(&mut self, e: &Expr) -> Result<Type, String> {
        // println!("{:?}", self.ctx);
        match e {
            // Rule 1l=>
            Expr::Unit => Ok(Type::Unit),
            // Rule Anno
            Expr::Ann(x, ty) => {
                self.check_wf(ty)?;
                self.check(x, ty)?;
                Ok(*ty.clone())
            }
            // Rule Var
            Expr::Var(x) => self.find_annotation(*x).cloned().ok_or(format!("unbound db {:?}", x)),
            // Rule ->I =>
            Expr::Abs(e) => {
                let alpha = self.fresh_ev();
                let beta = self.fresh_ev();
                // Fresh existential var for function domain
                self.ctx.push(Element::Exist(alpha));
                // And for codomain
                self.ctx.push(Element::Exist(beta));

                // Check the function body against Beta
                self.with_scope(Element::Ann(Type::Exist(alpha)), |f| f.check(e, &Type::Exist(beta)))?;

                // alpha and beta stay on the stack, since they appear in the output type
                Ok(Type::Arrow(Box::new(Type::Exist(alpha)), Box::new(Type::Exist(beta))))
            }
            // Rule ->E
            Expr::App(e1, e2) => {
                let a = self.infer(&e1)?;
                let a = self.apply(a);
                self.infer_app(&a, e2)
            }
        }
    }

    fn infer_app(&mut self, ty: &Type, e2: &Expr) -> Result<Type, String> {
        match ty {
            // Rule alpha_hat App
            Type::Exist(alpha) => {
                let a1 = self.fresh_ev();
                let a2 = self.fresh_ev();

                self.splice_hole(*alpha, |ctx| {
                    ctx.push(Element::Exist(a2));
                    ctx.push(Element::Exist(a1));
                    ctx.push(Element::Solved(
                        *alpha,
                        Type::Arrow(Box::new(Type::Exist(a1)), Box::new(Type::Exist(a2))),
                    ));
                })?;

                self.check(e2, &Type::Exist(a1))?;
                Ok(Type::Exist(a2))
            }
            // Rule ->App
            Type::Arrow(a, b) => {
                self.check(e2, a)?;
                Ok(*b.clone())
            }
            // Rule forall. App
            Type::Univ(a) => {
                let alpha = self.fresh_ev();
                let mut a_prime = a.clone();
                a_prime.subst(&Type::Exist(alpha));
                self.ctx.push(Element::Exist(alpha));
                self.infer_app(&a_prime, e2)
            }
            _ => Err(format!("Cannot appl ty {:?} to expr {:?}", e2, ty)),
        }
    }

    fn check(&mut self, e: &Expr, a: &Type) -> Result<(), String> {
        match (e, a) {
            // Rule 1l
            (Expr::Unit, Type::Unit) => Ok(()),
            // Rule ->I
            (Expr::Abs(body), Type::Arrow(a1, a2)) => self.with_scope(Element::Ann(*a1.clone()), |f| f.check(body, a2)),
            // Rule forall. I
            (e, Type::Univ(ty)) => self.with_scope(Element::Var, |f| f.check(e, &ty)),
            // Rule Sub
            (e, b) => {
                let a = self.infer(e)?;
                let a = self.apply(a);
                let b = self.apply(b.clone());
                self.subtype(&a, &b)?;
                Ok(())
            }
        }
    }
}

macro_rules! var {
    ($x:expr) => {
        Expr::Var($x)
    };
}

macro_rules! app {
    ($x:expr, $y:expr) => {
        Expr::App(Box::new($x), Box::new($y))
    };
}

macro_rules! abs {
    ($x:expr) => {
        Expr::Abs(Box::new($x))
    };
}

macro_rules! ann {
    ($x:expr, $t:expr) => {
        Expr::Ann(Box::new($x), Box::new($t))
    };
}

fn main() {
    println!("Hello, world!");

    // \f. \g. \x. f (g x)
    // : (e6 -> e7) -> ((e14 -> e6) -> (e14 -> e7))
    // : (a -> b) -> ((c -> a) -> (c -> b))
    let t1 = abs!(abs!(abs!(app!(var!(2), app!(var!(1), var!(0))))));
    let id = abs!(var!(0));
    // let id = ann!(id, Type::Univ(Box::new(Type::Arrow(Box::new(Type::Var(0)), Box::new(Type::Var(0))))));

    let mut ctx = Context::default();

    let inf = ctx.infer(&t1).unwrap();
    // dbg!(&inf);
    println!("final {:?}\n{:?}", &ctx.ctx, inf);
    let app = ctx.apply(inf);
    println!("typed as {:?}", &app);
}
