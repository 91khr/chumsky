//! TODO

use super::*;
use alloc::collections::LinkedList;
use hashbrown::HashSet;

/// A utility trait for types that can be constructed from a series of items.
pub trait Container<T>: Default {
    /// Create a container, attempting to pre-allocate enough space for `n` items.
    ///
    /// Failure to do so is not a problem, the size is only a hint.
    fn with_capacity(n: usize) -> Self {
        let _ = n;
        Self::default()
    }
    /// Add a value to the end of this container.
    fn push(&mut self, item: T);
}

impl<T> Container<T> for () {
    fn push(&mut self, _: T) {}
}

/// A collection that counts items instead of containing them.
impl<T> Container<T> for usize {
    fn push(&mut self, _: T) {
        *self += 1;
    }
}

impl<T> Container<T> for Vec<T> {
    fn with_capacity(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn push(&mut self, item: T) {
        (*self).push(item);
    }
}

impl<T> Container<T> for LinkedList<T> {
    fn push(&mut self, item: T) {
        (*self).push_back(item);
    }
}

impl Container<char> for String {
    fn with_capacity(n: usize) -> Self {
        // Note: we're assuming that most characters are going to be ASCII, and hence only require one byte to store.
        Self::with_capacity(n)
    }
    fn push(&mut self, item: char) {
        (*self).push(item)
    }
}

impl<K: Eq + Hash, V> Container<(K, V)> for HashMap<K, V> {
    fn with_capacity(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn push(&mut self, (key, value): (K, V)) {
        (*self).insert(key, value);
    }
}

#[cfg(feature = "std")]
impl<K: Eq + Hash, V> Container<(K, V)> for std::collections::HashMap<K, V> {
    fn with_capacity(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn push(&mut self, (key, value): (K, V)) {
        (*self).insert(key, value);
    }
}

impl<T: Eq + Hash> Container<T> for HashSet<T> {
    fn with_capacity(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn push(&mut self, item: T) {
        (*self).insert(item);
    }
}

#[cfg(feature = "std")]
impl<T: Eq + Hash> Container<T> for std::collections::HashSet<T> {
    fn with_capacity(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn push(&mut self, item: T) {
        (*self).insert(item);
    }
}

impl<K: Ord, V> Container<(K, V)> for alloc::collections::BTreeMap<K, V> {
    fn push(&mut self, (key, value): (K, V)) {
        (*self).insert(key, value);
    }
}

impl<T: Ord> Container<T> for alloc::collections::BTreeSet<T> {
    fn push(&mut self, item: T) {
        (*self).insert(item);
    }
}

/// A utility trait for types that hold a specific constant number of output values.
pub trait ContainerExactly<T, const N: usize> {
    /// An uninitialized value of this container.
    type Uninit;

    /// Get an uninitialized form of this container.
    fn uninit() -> Self::Uninit;

    /// Write a value to a position in an uninitialized container.
    fn write(uninit: &mut Self::Uninit, i: usize, item: T);

    /// Drop all values before a provided index in this container.
    ///
    /// # Safety
    ///
    /// All values up to the provided index must be initialized.
    unsafe fn drop_before(uninit: &mut Self::Uninit, i: usize);

    /// Convert this container into its initialized form.
    ///
    /// # Safety
    ///
    /// All values in the container must be initialized.
    unsafe fn take(uninit: Self::Uninit) -> Self;
}

impl<T, const N: usize> ContainerExactly<T, N> for () {
    type Uninit = ();
    fn uninit() -> Self::Uninit {}
    fn write(_: &mut Self::Uninit, _: usize, _: T) {}
    unsafe fn drop_before(_: &mut Self::Uninit, _: usize) {}
    unsafe fn take(_: Self::Uninit) -> Self {}
}

impl<T, const N: usize> ContainerExactly<T, N> for [T; N] {
    type Uninit = [MaybeUninit<T>; N];
    fn uninit() -> Self::Uninit {
        MaybeUninitExt::uninit_array()
    }
    fn write(uninit: &mut Self::Uninit, i: usize, item: T) {
        uninit[i].write(item);
    }
    unsafe fn drop_before(uninit: &mut Self::Uninit, i: usize) {
        uninit[..i].iter_mut().for_each(|o| o.assume_init_drop());
    }
    unsafe fn take(uninit: Self::Uninit) -> Self {
        MaybeUninitExt::array_assume_init(uninit)
    }
}

/// A utility trait to abstract over container-like things.
///
/// This trait is likely to change in future versions of the crate, so avoid implementing it yourself.
pub trait Seq<'p, T> {
    /// The item yielded by the iterator.
    type Item<'a>: Borrow<T>
    where
        Self: 'a;

    /// An iterator over the items within this container, by reference.
    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        Self: 'a;

    /// Iterate over the elements of the container.
    fn seq_iter(&self) -> Self::Iter<'_>;

    /// Check whether an item is contained within this sequence.
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq;

    /// Convert an item of the sequence into a [`MaybeRef`].
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b;
}

impl<'p, T: Clone> Seq<'p, T> for T {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = core::iter::Once<&'a T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        core::iter::once(self)
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        self == val
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T> Seq<'p, T> for &'p T {
    type Item<'a> = &'p T
    where
        Self: 'a;

    type Iter<'a> = core::iter::Once<&'p T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        core::iter::once(*self)
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        *self == val
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Ref(item)
    }
}

impl<'p, T> Seq<'p, T> for &'p [T] {
    type Item<'a> = &'p T
    where
        Self: 'a;

    type Iter<'a> = core::slice::Iter<'p, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        (self as &[T]).iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        <[T]>::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Ref(item)
    }
}

impl<'p, T: Clone, const N: usize> Seq<'p, T> for [T; N] {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        <[T]>::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T, const N: usize> Seq<'p, T> for &'p [T; N] {
    type Item<'a> = &'p T
    where
        Self: 'a;

    type Iter<'a> = core::slice::Iter<'p, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        <[T]>::contains(*self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Ref(item)
    }
}

impl<'p, T: Clone> Seq<'p, T> for Vec<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        <[T]>::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T: Clone> Seq<'p, T> for LinkedList<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = alloc::collections::linked_list::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        LinkedList::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T: Clone + Eq + Hash> Seq<'p, T> for HashSet<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = hashbrown::hash_set::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        HashSet::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

#[cfg(feature = "std")]
impl<'p, T: Clone + Eq + Hash> Seq<'p, T> for std::collections::HashSet<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = std::collections::hash_set::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        self.contains(val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T: Clone + Ord> Seq<'p, T> for alloc::collections::BTreeSet<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    type Iter<'a> = alloc::collections::btree_set::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        self.contains(val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item.clone())
    }
}

impl<'p, T> Seq<'p, T> for Range<T>
where
    T: Clone + PartialOrd, // Explicit declaration of an implied truth - `Step` requires these
    Self: Iterator<Item = T>,
{
    type Item<'a> = T
    where
        Self: 'a;

    type Iter<'a> = Range<T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        (*self).clone()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool {
        Range::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

impl<'p, T> Seq<'p, T> for core::ops::RangeInclusive<T>
where
    T: Clone + PartialOrd,
    Self: Iterator<Item = T>,
{
    type Item<'a> = T
    where
        Self: 'a;

    type Iter<'a> = core::ops::RangeInclusive<T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.clone()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool {
        core::ops::RangeInclusive::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

impl<'p, T> Seq<'p, T> for RangeFrom<T>
where
    T: Clone + PartialOrd,
    Self: Iterator<Item = T>,
{
    type Item<'a> = T
    where
        Self: 'a;

    type Iter<'a> = RangeFrom<T>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.clone()
    }

    #[inline(always)]
    fn contains(&self, val: &T) -> bool {
        RangeFrom::contains(self, val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, T>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

impl<'p> Seq<'p, char> for str {
    type Item<'a> = char
    where
        Self: 'a;

    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.chars()
    }

    #[inline(always)]
    fn contains(&self, val: &char) -> bool {
        self.contains(*val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, char>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

impl<'p> Seq<'p, char> for &'p str {
    type Item<'a> = char
    where
        Self: 'a;

    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.chars()
    }

    #[inline(always)]
    fn contains(&self, val: &char) -> bool {
        str::contains(self, *val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, char>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

impl<'p> Seq<'p, char> for String {
    type Item<'a> = char
    where
        Self: 'a;

    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn seq_iter(&self) -> Self::Iter<'_> {
        self.chars()
    }

    #[inline(always)]
    fn contains(&self, val: &char) -> bool {
        str::contains(self, *val)
    }

    #[inline]
    fn to_maybe_ref<'b>(item: Self::Item<'b>) -> MaybeRef<'p, char>
    where
        'p: 'b,
    {
        MaybeRef::Val(item)
    }
}

/// A utility trait to abstract over *linear* container-like things.
///
/// This trait is likely to change in future versions of the crate, so avoid implementing it yourself.
pub trait OrderedSeq<'p, T>: Seq<'p, T> {}

impl<'p, T: Clone> OrderedSeq<'p, T> for T {}
impl<'p, T> OrderedSeq<'p, T> for &'p T {}
impl<'p, T> OrderedSeq<'p, T> for &'p [T] {}
impl<'p, T: Clone, const N: usize> OrderedSeq<'p, T> for [T; N] {}
impl<'p, T, const N: usize> OrderedSeq<'p, T> for &'p [T; N] {}
impl<'p, T: Clone> OrderedSeq<'p, T> for Vec<T> {}
impl<'p, T> OrderedSeq<'p, T> for Range<T> where Self: Seq<'p, T> {}
impl<'p, T> OrderedSeq<'p, T> for core::ops::RangeInclusive<T> where Self: Seq<'p, T> {}
impl<'p, T> OrderedSeq<'p, T> for RangeFrom<T> where Self: Seq<'p, T> {}

impl<'p> OrderedSeq<'p, char> for str {}
impl<'p> OrderedSeq<'p, char> for &'p str {}
impl<'p> OrderedSeq<'p, char> for String {}
