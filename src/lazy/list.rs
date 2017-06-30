use std::ops::{Add, Deref};
use std::borrow::Borrow;
use std::sync::Arc;
use std::iter::FromIterator;
use std::cmp::Ordering;
use thunk::{ArcThunk, LazyRef};

use self::Step::{Nil, Cons};

pub struct List<A>(ArcThunk<Step<A>>);

pub enum Step<A> {
    Nil,
    Cons(Arc<A>, List<A>),
}

impl<A> List<A> {
    pub fn new() -> Self {
        List::computed(Nil)
    }

    pub fn cons<R>(&self, car: R) -> Self
    where
        Arc<A>: From<R>,
    {
        List::computed(Cons(From::from(car), self.clone()))
    }

    pub fn head(&self) -> Option<Arc<A>> {
        match self.0.deref() {
            &Nil => None,
            &Cons(ref car, _) => Some(car.clone()),
        }
    }

    pub fn tail(&self) -> Option<List<A>> {
        match self.0.deref() {
            &Nil => None,
            &Cons(_, ref cdr) => Some(cdr.clone()),
        }
    }

    pub fn uncons(&self) -> Option<(Arc<A>, List<A>)> {
        match self.0.deref() {
            &Nil => None,
            &Cons(ref car, ref cdr) => Some((car.clone(), cdr.clone())),
        }
    }

    pub fn iter(&self) -> Iter<A> {
        Iter::new(self)
    }

    pub fn append<R>(&self, other: R) -> Self where R: Borrow<Self> {
        List::from_iter(self.iter().chain(other.borrow().iter()))
    }

    pub fn unfold<F, B, R>(value: B, f: F) -> List<A>
    where
        F: Fn(B) -> Option<(R, B)>,
        Arc<A>: From<R>,
    {
        List::defer(|| match f(value) {
            None => Nil,
            Some((a, b)) => Cons(From::from(a), List::unfold(b, f)),
        })
    }
}

// Traits

impl<A> Clone for List<A> {
    fn clone(&self) -> Self {
        List(self.0.clone())
    }
}

impl<A> Default for List<A> {
    fn default() -> Self {
        List::new()
    }
}

impl<A: PartialEq> PartialEq for List<A> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<A: Eq> Eq for List<A> {}

impl<A: PartialOrd> PartialOrd for List<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<A: Ord> Ord for List<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<'a, A> Add for &'a List<A> {
    type Output = List<A>;

    fn add(self, other: Self) -> Self::Output {
        self.append(other)
    }
}

impl<A> Add for List<A> {
    type Output = List<A>;

    fn add(self, other: Self) -> Self::Output {
        self.append(&other)
    }
}

impl<A> Deref for List<A> {
    type Target = Step<A>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<A> From<Step<A>> for List<A> {
    fn from(step: Step<A>) -> Self {
        List(ArcThunk::defer(|| step))
    }
}

impl<A> LazyRef for List<A> {
    fn defer<'a, F: FnOnce() -> Step<A> + 'a>(f: F) -> Self {
        List(ArcThunk::defer(f))
    }

    fn force(&self) {
        self.0.force()
    }
}

impl<R, A> FromIterator<R> for List<A>
where
    Arc<A>: From<R>,
{
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = R>,
    {
        List::unfold(
            i.into_iter(),
            |mut it| it.next().map(|a| (From::from(a), it)),
        )
    }
}

impl<'a, A> IntoIterator for &'a List<A> {
    type Item = Arc<A>;
    type IntoIter = Iter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> IntoIterator for List<A> {
    type Item = Arc<A>;
    type IntoIter = Iter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<A> {
    current: List<A>,
}

impl<A> Iter<A> {
    fn new(list: &List<A>) -> Iter<A> {
        Iter { current: list.clone() }
    }
}

impl<A> Iterator for Iter<A> {
    type Item = Arc<A>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.uncons() {
            None => None,
            Some((car, cdr)) => {
                self.current = cdr;
                Some(car)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn infinite_list() {
        let mut it = List::unfold(0, |n| Some((n, n + 1))).iter();
        for i in 0..100000 {
            assert_eq!(Some(Arc::new(i)), it.next())
        }
    }
}
